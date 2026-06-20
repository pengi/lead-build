mod export;
pub use export::Exportable;

#[cfg(test)]
mod tests;

use std::{
    cell::{Ref, RefCell},
    collections::BTreeMap,
    fmt::{Debug, Display},
    rc::Rc,
};

pub mod ops {
    use super::{Debug, Display, Expr};

    pub trait ExprOps: Sized {
        fn op_add(lhs: &Self, rhs: &Self) -> Result<Self>;
        fn op_sub(lhs: &Self, rhs: &Self) -> Result<Self>;
        fn op_mult(lhs: &Self, rhs: &Self) -> Result<Self>;
        fn op_div(lhs: &Self, rhs: &Self) -> Result<Self>;
        fn op_lt(lhs: &Self, rhs: &Self) -> Result<Self>;
        fn op_le(lhs: &Self, rhs: &Self) -> Result<Self>;
        fn op_gt(lhs: &Self, rhs: &Self) -> Result<Self>;
        fn op_ge(lhs: &Self, rhs: &Self) -> Result<Self>;
        fn op_eq(lhs: &Self, rhs: &Self) -> Result<Self>;
        fn op_neq(lhs: &Self, rhs: &Self) -> Result<Self>;
        fn op_neg(&self) -> Result<Self>;
        fn op_not(&self) -> Result<Self>;
        fn as_bool(&self) -> Result<bool>;
        fn as_string(&self) -> Result<String>;
        fn new_from_bool(&self, value: bool) -> Self;
    }

    pub trait ExprBuiltin<T, F>: Debug
    where
        T: Clone + PartialEq + Display + ExprOps,
        F: Clone,
    {
        fn get_name(&self) -> String;
        fn call(&self, arg: Expr<T, F>) -> Result<Expr<T, F>>;
    }

    pub enum Error {
        Type(String),
        ExprError(super::Error),
    }

    impl From<super::Error> for Error {
        fn from(value: super::Error) -> Self {
            Error::ExprError(value)
        }
    }

    impl From<super::super::error::Error> for Error {
        fn from(value: super::super::error::Error) -> Self {
            Error::ExprError(value.into())
        }
    }

    pub type Result<T> = std::result::Result<T, Error>;
}

use ops::{ExprBuiltin, ExprOps};
use strum::EnumTryAs;

/*
 * Error
 */

#[derive(Debug)]
pub enum Error {
    Scope(String),
    Eval(String),
    Type(String),
    DupKey(String),
    NoValue(String),
    Lang(Rc<super::error::Error>),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Scope(msg) => write!(f, "ScopeError: {}", msg),
            Error::Eval(msg) => write!(f, "EvalError: {}", msg),
            Error::Type(msg) => write!(f, "TypeError: {}", msg),
            Error::DupKey(msg) => write!(f, "DupKey: {}", msg),
            Error::NoValue(msg) => write!(f, "No value: {}", msg),
            Error::Lang(dnj_error) => Display::fmt(&dnj_error, f),
        }
    }
}

impl From<ops::Error> for Error {
    fn from(value: ops::Error) -> Self {
        match value {
            ops::Error::Type(msg) => Error::Type(msg),
            ops::Error::ExprError(err) => err,
        }
    }
}

impl From<super::error::Error> for Error {
    fn from(value: super::error::Error) -> Self {
        Error::Lang(value.into())
    }
}

type Result<RT> = std::result::Result<RT, Error>;

/* *****************************************************************************
 * Types
 */

#[derive(Debug, PartialEq, Clone)]
pub struct Expr<T, F>(Rc<ExprStorage<T, F>>)
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone;

// TODO: Better implementation of ExprSet... This probably takes time to clone.
pub type ExprSet<T, F> = BTreeMap<String, Expr<T, F>>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ExprBinOp {
    HasAttr,
    ListConcat,
    Mult,
    Div,
    Sub,
    Add,
    Update,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Neq,
    LogAnd,
    LogOr,
    LogImpl,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ExprUnOp {
    Neg,
    Not,
}

#[derive(Clone)]
pub struct ExprBuiltinWrapper<T, F>(String, Rc<dyn ExprBuiltin<T, F>>)
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone;

#[derive(Debug, PartialEq, Clone)]
pub struct ExprSourceRef<F> {
    file: F,
    left: usize,
    right: usize,
}

#[derive(Debug, Clone)]
pub struct ExprStorage<T, F>
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone,
{
    tok: RefCell<ExprType<T, F>>,
    loc: Option<ExprSourceRef<F>>,
}

// Clone is needed since ExprType::Var is implemented via cloning of ExprType
#[derive(Debug, PartialEq, Clone, Default, EnumTryAs)]
pub enum ExprType<T, F>
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone,
{
    Object(ExprSet<T, F>),
    List(Vec<Expr<T, F>>),
    AttrSel(Expr<T, F>, String),
    Value(T),
    Var(String),
    UnOp(ExprUnOp, Expr<T, F>),
    BinOp(ExprBinOp, Expr<T, F>, Expr<T, F>),
    FuncDefIdent(String, Expr<T, F>),
    FuncDefPattern(Vec<String>, Expr<T, F>),
    FuncDefBuiltin(ExprBuiltinWrapper<T, F>),
    Let(Vec<(String, Expr<T, F>)>, Expr<T, F>),
    MapList(Expr<T, F>, Expr<T, F>),
    FuncCall(Expr<T, F>, Expr<T, F>),
    Bind(ExprSet<T, F>, Expr<T, F>),
    #[default]
    Null,
}

/* *****************************************************************************
 * PartialEq
 */

impl<T, F> PartialEq for ExprStorage<T, F>
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.tok == other.tok
    }
}

/* *****************************************************************************
 * Location handling
 */

impl<T, F> Expr<T, F>
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone,
{
    pub fn get_loc(&self) -> Option<ExprSourceRef<F>> {
        self.0.as_ref().loc.clone()
    }
}

impl<T, F> ExprType<T, F>
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone,
{
    pub fn reref(self: ExprType<T, F>, loc: Option<ExprSourceRef<F>>) -> Expr<T, F> {
        Expr(Rc::new(ExprStorage {
            tok: RefCell::new(self),
            loc,
        }))
    }

    pub fn loc(self: ExprType<T, F>, left: usize, right: usize, f: &F) -> Expr<T, F> {
        self.reref(Some(ExprSourceRef {
            file: f.clone(),
            left,
            right,
        }))
    }

    pub fn builtin(self: ExprType<T, F>) -> Expr<T, F> {
        self.reref(None)
    }
}

/* *****************************************************************************
 * Display
 */

impl<T, F> Debug for ExprBuiltinWrapper<T, F>
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ExprBuiltinWrapper").field(&self.0).finish()
    }
}

impl Display for ExprBinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprBinOp::HasAttr => write!(f, "?"),
            ExprBinOp::ListConcat => write!(f, "++"),
            ExprBinOp::Mult => write!(f, "*"),
            ExprBinOp::Div => write!(f, "/"),
            ExprBinOp::Sub => write!(f, "-"),
            ExprBinOp::Add => write!(f, "+"),
            ExprBinOp::Update => write!(f, "//"),
            ExprBinOp::Lt => write!(f, "<"),
            ExprBinOp::Le => write!(f, "<="),
            ExprBinOp::Gt => write!(f, ">"),
            ExprBinOp::Ge => write!(f, ">="),
            ExprBinOp::Eq => write!(f, "=="),
            ExprBinOp::Neq => write!(f, "!="),
            ExprBinOp::LogAnd => write!(f, "&&"),
            ExprBinOp::LogOr => write!(f, "||"),
            ExprBinOp::LogImpl => write!(f, "->"),
        }
    }
}

impl Display for ExprUnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprUnOp::Neg => write!(f, "-"),
            ExprUnOp::Not => write!(f, "!"),
        }
    }
}

impl<T, F> Display for Expr<T, F>
where
    T: Clone + PartialEq + Display + ExprOps + Debug + Exportable,
    F: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.export(0, f)
    }
}

impl<T, F> Display for ExprType<T, F>
where
    T: Clone + PartialEq + Display + ExprOps + Debug + Exportable,
    F: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.export(0, f)
    }
}

/* *****************************************************************************
 * Transform / From
 */

impl<T, F> From<ExprType<T, F>> for ExprStorage<T, F>
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone,
{
    fn from(value: ExprType<T, F>) -> Self {
        ExprStorage {
            tok: RefCell::new(value),
            loc: None, // TODO
        }
    }
}

impl<T, F> From<ExprStorage<T, F>> for Expr<T, F>
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone,
{
    fn from(value: ExprStorage<T, F>) -> Self {
        Expr(Rc::new(value))
    }
}

impl<T, F> From<ExprType<T, F>> for Expr<T, F>
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone,
{
    fn from(value: ExprType<T, F>) -> Self {
        Expr::from(ExprStorage::from(value))
    }
}

impl<T, F> From<ExprSet<T, F>> for Expr<T, F>
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone,
{
    fn from(value: ExprSet<T, F>) -> Self {
        Expr::from(ExprType::Object(value))
    }
}

impl<T, F> From<T> for ExprType<T, F>
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone,
{
    fn from(value: T) -> Self {
        ExprType::Value(value)
    }
}

/* *****************************************************************************
 * Implementations
 */

impl<T, F> PartialEq for ExprBuiltinWrapper<T, F>
where
    T: Clone + PartialEq + Display + ExprOps,
    F: Clone,
{
    fn eq(&self, other: &Self) -> bool {
        #[cfg(test)]
        {
            self.0 == other.0
        }
        #[cfg(not(test))]
        {
            let _ = other;
            unreachable!("== of builtin expressions should not be used")
        }
    }
}

impl<T, F> Expr<T, F>
where
    T: Clone + PartialEq + Display + ExprOps + Debug + Exportable,
    F: Clone,
{
    pub fn inner_ref(&self) -> Ref<'_, ExprType<T, F>> {
        self.0.as_ref().tok.borrow()
    }

    pub fn resolve(&self) -> Result<()> {
        let mut expr = self.0.as_ref().tok.take();

        while match &expr {
            ExprType::Object(..) => false,
            ExprType::List(..) => false,
            ExprType::AttrSel(..) => true,
            ExprType::Value(..) => false,
            ExprType::Var(..) => true,
            ExprType::UnOp(..) => true,
            ExprType::BinOp(..) => true,
            ExprType::FuncDefIdent(..) => false,
            ExprType::FuncDefPattern(..) => false,
            ExprType::FuncDefBuiltin(..) => false,
            ExprType::Let(..) => true,
            ExprType::MapList(..) => true,
            ExprType::FuncCall(..) => true,
            ExprType::Bind(..) => true,
            ExprType::Null => false,
        } {
            expr = match expr {
                ExprType::Bind(varspace, bound_expr) => match &*bound_expr.inner_ref() {
                    ExprType::Object(fields) => Ok(ExprType::Object(
                        fields
                            .iter()
                            .map(|(k, val)| {
                                (
                                    k.clone(),
                                    ExprType::Bind(varspace.clone(), val.clone()).into(),
                                )
                            })
                            .collect(),
                    )),
                    ExprType::List(items) => Ok(ExprType::List(
                        items
                            .iter()
                            .map(|item| ExprType::Bind(varspace.clone(), item.clone()).into())
                            .collect(),
                    )),
                    ExprType::AttrSel(val, attr) => Ok(ExprType::AttrSel(
                        ExprType::Bind(varspace, val.clone()).into(),
                        attr.clone(),
                    )),
                    ExprType::Let(fields, target_expr) => {
                        let mut vars: ExprSet<T, F> = varspace;
                        for (field_name, field_expr) in fields {
                            let field_vars = vars.clone();
                            vars.insert(
                                field_name.clone(),
                                ExprType::Bind(field_vars, field_expr.clone()).into(),
                            )
                            .map_or_else(|| Ok(()), |_| Err(Error::DupKey(field_name.clone())))?;
                        }
                        Ok(ExprType::Bind(vars, target_expr.clone()))
                    }
                    ExprType::FuncDefIdent(arg_name, func_expr) => {
                        let mut new_scope = varspace;
                        new_scope.remove(arg_name);
                        Ok(ExprType::FuncDefIdent(
                            arg_name.clone(),
                            ExprType::Bind(new_scope, func_expr.clone()).into(),
                        ))
                    }
                    ExprType::FuncDefPattern(items, expr) => {
                        let mut new_scope = varspace;
                        for item in items.iter() {
                            new_scope.remove(item);
                        }
                        Ok(ExprType::FuncDefPattern(
                            items.clone(),
                            ExprType::Bind(new_scope, expr.clone()).into(),
                        ))
                    }
                    ExprType::FuncDefBuiltin(_expr_builtin) => todo!(),
                    ExprType::MapList(func, input) => Ok(ExprType::MapList(
                        ExprType::Bind(varspace.clone(), func.clone()).into(),
                        ExprType::Bind(varspace.clone(), input.clone()).into(),
                    )),
                    ExprType::Var(name) => match &varspace.get(name) {
                        Some(value) => Ok(value.res_type()?.clone()),
                        None => Err(Error::Scope(format!("Unknown variable {}", name))),
                    },
                    ExprType::UnOp(op, expr) => Ok(ExprType::UnOp(
                        *op,
                        ExprType::Bind(varspace, expr.clone()).into(),
                    )),
                    ExprType::BinOp(op, lhs, rhs) => Ok(ExprType::BinOp(
                        *op,
                        ExprType::Bind(varspace.clone(), lhs.clone()).into(),
                        ExprType::Bind(varspace, rhs.clone()).into(),
                    )),
                    ExprType::FuncCall(fexpr, fargs) => Ok(ExprType::FuncCall(
                        ExprType::Bind(varspace.clone(), fexpr.clone()).into(),
                        ExprType::Bind(varspace, fargs.clone()).into(),
                    )),
                    ExprType::Value(value) => Ok(ExprType::Value(value.clone())),
                    ExprType::Bind(inner_vars, inner_expr) => {
                        Ok(ExprType::Bind(inner_vars.clone(), inner_expr.clone()))
                    }
                    ExprType::Null => panic!("Found null in expr tree"),
                },
                ExprType::AttrSel(val, attr) => {
                    let attr_expr = val.get_item(attr.as_str())?;
                    Ok(attr_expr.inner_ref().clone())
                }
                ExprType::FuncCall(fexpr, fargs) => {
                    let (mut args, func_expr): (ExprSet<T, F>, Expr<T, F>) =
                        match &*fexpr.res_type()? {
                            ExprType::FuncDefIdent(arg_name, fimpl) => Ok((
                                ExprSet::from([(arg_name.clone(), fargs.clone())]),
                                fimpl.clone(),
                            )),
                            ExprType::FuncDefPattern(arg_names, fimpl) => {
                                fargs.resolve()?;
                                let mut new_vars = ExprSet::new();
                                for arg_name in arg_names {
                                    let arg_value = fargs.get_item(arg_name)?;
                                    new_vars.insert(arg_name.clone(), arg_value).map_or_else(
                                        || Ok(()),
                                        |_| Err(Error::DupKey(arg_name.clone())),
                                    )?;
                                }
                                Ok((new_vars, fimpl.clone()))
                            }
                            ExprType::FuncDefBuiltin(ExprBuiltinWrapper(_, funcrc)) => {
                                let res = funcrc.as_ref().call(fargs)?;
                                Ok((ExprSet::new(), res))
                            }
                            _ => Err(Error::Scope(format!("called func, but it's a {}", fexpr))),
                        }?;

                    // If function contains a bound scope, it should still apply,
                    // and not overwrite input arguments.
                    match &*func_expr.inner_ref() {
                        ExprType::Bind(varspace, inner_expr) => {
                            let mut merged_varspace = varspace.clone();
                            merged_varspace.append(&mut args);
                            Ok(ExprType::Bind(merged_varspace, inner_expr.clone()))
                        }
                        _ => Ok(ExprType::Bind(args, func_expr.clone())),
                    }
                }
                ExprType::MapList(func, input) => {
                    input.resolve()?;
                    match &*input.inner_ref() {
                        ExprType::List(input_vec) => Ok(ExprType::List(
                            input_vec
                                .iter()
                                .map(|iel| ExprType::FuncCall(func.clone(), iel.clone()).into())
                                .collect::<Vec<_>>(),
                        )),
                        _ => Err(Error::Eval(format!("Foreach over non-list: {}", input))),
                    }
                }
                ExprType::UnOp(op, expr) => {
                    expr.resolve()?;
                    match op {
                        ExprUnOp::Neg => match &*expr.inner_ref() {
                            ExprType::Value(value) => Ok(ExprType::Value(value.op_neg()?)),
                            _ => Err(Error::Eval(format!("negating non-value: {}", expr))),
                        },
                        ExprUnOp::Not => match &*expr.inner_ref() {
                            ExprType::Value(value) => Ok(ExprType::Value(value.op_not()?)),
                            _ => Err(Error::Eval(format!("negating non-value: {}", expr))),
                        },
                    }
                }
                ExprType::BinOp(op, lhs, rhs) => match &*lhs.res_type()? {
                    ExprType::Object(_lhs_obj) => todo!("Binop on object"),
                    ExprType::List(lhs_list) => match (op, &*rhs.res_type()?) {
                        (ExprBinOp::Add, ExprType::List(rhs_list)) => {
                            let mut res = lhs_list.clone();
                            res.extend(rhs_list.iter().cloned());
                            Ok(ExprType::List(res))
                        }
                        _ => todo!("error message"),
                    },
                    ExprType::Value(lhs_val) => match op {
                        ExprBinOp::LogAnd => match lhs_val.as_bool()? {
                            true => Ok(rhs.res_type()?.clone()),
                            false => Ok(ExprType::Value(lhs_val.new_from_bool(false))),
                        },
                        ExprBinOp::LogOr => match lhs_val.as_bool()? {
                            true => Ok(ExprType::Value(lhs_val.new_from_bool(true))),
                            false => Ok(rhs.res_type()?.clone()),
                        },
                        ExprBinOp::LogImpl => match lhs_val.as_bool()? {
                            false => Ok(ExprType::Value(lhs_val.new_from_bool(true))),
                            true => Ok(rhs.res_type()?.clone()),
                        },
                        _ => match &*rhs.res_type()? {
                            ExprType::Object(_rhs_obj) => todo!(),
                            ExprType::Value(rhs_val) => match op {
                                ExprBinOp::HasAttr => todo!(),
                                ExprBinOp::ListConcat => todo!(),
                                ExprBinOp::Mult => {
                                    Ok(ExprType::Value(T::op_mult(lhs_val, rhs_val)?))
                                }
                                ExprBinOp::Div => Ok(ExprType::Value(T::op_div(lhs_val, rhs_val)?)),
                                ExprBinOp::Sub => Ok(ExprType::Value(T::op_sub(lhs_val, rhs_val)?)),
                                ExprBinOp::Add => Ok(ExprType::Value(T::op_add(lhs_val, rhs_val)?)),
                                ExprBinOp::Update => todo!(),
                                ExprBinOp::Lt => Ok(ExprType::Value(T::op_lt(lhs_val, rhs_val)?)),
                                ExprBinOp::Le => Ok(ExprType::Value(T::op_le(lhs_val, rhs_val)?)),
                                ExprBinOp::Gt => Ok(ExprType::Value(T::op_gt(lhs_val, rhs_val)?)),
                                ExprBinOp::Ge => Ok(ExprType::Value(T::op_ge(lhs_val, rhs_val)?)),
                                ExprBinOp::Eq => Ok(ExprType::Value(T::op_eq(lhs_val, rhs_val)?)),
                                ExprBinOp::Neq => Ok(ExprType::Value(T::op_neq(lhs_val, rhs_val)?)),
                                _ => unreachable!(),
                            },
                            typ => Err(Error::Eval(format!("Resolving unresolvable type {}", typ))),
                        },
                    },
                    typ => Err(Error::Eval(format!("Resolving unresolvable type {}", typ))),
                },
                ExprType::Null => panic!("Found null in expr tree"),
                typ => unreachable!("Resolving {}", typ),
            }?;
        }

        self.0.as_ref().tok.replace(expr);
        Ok(())
    }

    fn res_type(&self) -> Result<Ref<'_, ExprType<T, F>>> {
        self.resolve()?;
        Ok(self.inner_ref())
    }

    pub fn eval(&self) -> Result<()> {
        self.resolve()?;
        match &*self.inner_ref() {
            ExprType::Object(fields) => {
                for (_, field) in fields.iter() {
                    field.eval()?;
                }
            }
            ExprType::List(fields) => {
                for ex in fields.iter() {
                    ex.eval()?
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn value(&self) -> Result<T> {
        // Since we expect a string, we only need to resolve one level.
        self.resolve()?;
        match &*self.inner_ref() {
            ExprType::Value(val) => Ok(val.clone()),
            _ => Err(Error::NoValue(format!("Not a value: {}", self))),
        }
    }

    pub fn eval_string(&self) -> Result<String> {
        // Since we expect a string, we only need to resolve one level.
        self.resolve()?;
        match &*self.inner_ref() {
            ExprType::Value(val) => Ok(val.as_string()?),
            _ => Err(Error::NoValue(format!("Not a string: {}", self))),
        }
    }

    pub fn get_item(&self, name: &str) -> Result<Expr<T, F>> {
        self.resolve()?;
        let node = self.inner_ref();
        match &*node {
            ExprType::Object(vars) => Ok(vars
                .get(name)
                .ok_or_else(|| Error::NoValue(format!("Invalid field '{}'", name)))?
                .clone()),
            _ => Err(Error::NoValue(format!("Invalid item '{}'", name))),
        }
    }

    pub fn new_builtin(func: Rc<dyn ExprBuiltin<T, F>>) -> Expr<T, F> {
        ExprType::FuncDefBuiltin(ExprBuiltinWrapper(func.as_ref().get_name(), func)).into()
    }

    pub fn from_builtins(value: Vec<Rc<dyn ExprBuiltin<T, F>>>) -> Expr<T, F> {
        let mut exprset = ExprSet::new();

        for bi in value.into_iter() {
            let name = bi.get_name();
            exprset
                .insert(
                    name.clone(),
                    ExprType::FuncDefBuiltin(ExprBuiltinWrapper(name, bi)).into(),
                )
                .unwrap();
        }

        exprset.into()
    }
}
