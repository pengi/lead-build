use std::{
    cell::{Ref, RefCell},
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::immap::ImMap;

pub mod ops {
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
        fn new_from_bool(&self, value: bool) -> Self;
    }

    pub enum Error {
        Type(String),
    }

    pub type Result<T> = std::result::Result<T, Error>;
}

use ops::ExprOps;

/*
 * Error
 */

#[derive(Debug, PartialEq)]
pub enum Error {
    ScopeError(String),
    EvalError(String),
    TypeError(String),
    DupKey(String),
    NoValue(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ScopeError(msg) => write!(f, "ScopeError: {}", msg),
            Error::EvalError(msg) => write!(f, "EvalError: {}", msg),
            Error::TypeError(msg) => write!(f, "TypeError: {}", msg),
            Error::DupKey(msg) => write!(f, "DupKey: {}", msg),
            Error::NoValue(msg) => write!(f, "No value: {}", msg),
        }
    }
}

impl From<crate::immap::Error> for Error {
    fn from(value: crate::immap::Error) -> Self {
        match value {
            crate::immap::Error::DupKey(key) => Error::DupKey(key),
        }
    }
}

impl From<ops::Error> for Error {
    fn from(value: ops::Error) -> Self {
        match value {
            ops::Error::Type(msg) => Error::TypeError(msg),
        }
    }
}

type Result<RT> = std::result::Result<RT, Error>;

/* *****************************************************************************
 * Types
 */

#[derive(Debug, PartialEq, Clone)]
pub struct ExprRef<T>(Rc<RefCell<Expr<T>>>)
where
    T: Clone + PartialEq + Display + ExprOps;

pub type ExprSet<T> = ImMap<ExprRef<T>>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ExprBinOp {
    AttrSel,
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

#[derive(Debug, PartialEq, Clone)]
pub enum ExprType<T>
where
    T: Clone + PartialEq + Display + ExprOps,
{
    Object(ExprSet<T>),
    Value(T),
    Var(String),
    UnOp(ExprUnOp, ExprRef<T>),
    BinOp(ExprBinOp, ExprRef<T>, ExprRef<T>),
    FuncDefIdent(String, ExprRef<T>),
    FuncDefPattern(Vec<String>, ExprRef<T>),
    Let(Vec<(String, ExprRef<T>)>, ExprRef<T>),
    FuncCall(String, ExprRef<T>),
    CalledFunc(ExprRef<T>, ExprRef<T>),
    BoundExpr(ExprSet<T>, ExprRef<T>),
}

#[derive(Debug, PartialEq)]
pub struct Expr<T>(ExprType<T>)
where
    T: Clone + PartialEq + Display + ExprOps;

/* *****************************************************************************
 * Display
 */

impl Display for ExprBinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprBinOp::AttrSel => write!(f, "."),
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

impl<T> Display for ExprRef<T>
where
    T: Clone + PartialEq + Display + ExprOps,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_ref().borrow().fmt(f)
    }
}

impl<T> Display for ExprType<T>
where
    T: Clone + PartialEq + Display + ExprOps,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprType::Object(varscope) => varscope.fmt(f),
            ExprType::Value(val) => val.fmt(f),
            ExprType::Var(val) => Display::fmt(&val, f),
            ExprType::UnOp(op, expr) => {
                write!(f, "{}({})", op, expr)
            }
            ExprType::BinOp(op, lhs, rhs) => {
                write!(f, "({}){}({})", lhs, op, rhs)
            }
            ExprType::FuncDefIdent(name, expr) => write!(f, "{}: {}", name, expr),
            ExprType::FuncDefPattern(items, expr) => {
                f.write_str("{")?;
                for item in items {
                    Display::fmt(&item, f)?;
                    f.write_str(", ")?;
                }
                f.write_str("...}: ")?;
                expr.fmt(f)?;
                Ok(())
            }
            ExprType::Let(items, expr) => {
                f.write_str("let ")?;
                for (var_name, var_expr) in items {
                    std::fmt::Display::fmt(&var_name, f)?;
                    f.write_str("=")?;
                    std::fmt::Display::fmt(&var_expr, f)?;
                    f.write_str("; ")?;
                }
                f.write_str("in ")?;
                expr.fmt(f)?;
                Ok(())
            }
            ExprType::FuncCall(name, expr) => write!(f, "{} {}", name, expr),
            ExprType::CalledFunc(args, expr) => write!(f, "{} => {}", args, expr),
            ExprType::BoundExpr(scope, expr) => write!(f, "[ {} @ {} ]", scope, expr),
        }
    }
}

impl<T> Display for Expr<T>
where
    T: Clone + PartialEq + Display + ExprOps,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/* *****************************************************************************
 * Transform / From
 */

/*
 * Primitives
 */
impl<T> From<T> for ExprType<T>
where
    T: Clone + PartialEq + Display + ExprOps,
{
    fn from(value: T) -> Self {
        ExprType::Value(value)
    }
}

impl<T> From<ExprType<T>> for Expr<T>
where
    T: Clone + PartialEq + Display + ExprOps,
{
    fn from(value: ExprType<T>) -> Self {
        Expr(value)
    }
}

impl<T> From<Expr<T>> for ExprRef<T>
where
    T: Clone + PartialEq + Display + ExprOps,
{
    fn from(value: Expr<T>) -> Self {
        ExprRef(Rc::new(RefCell::new(value)))
    }
}

/*
 * Shortcuts
 */

impl<T> From<ExprType<T>> for ExprRef<T>
where
    T: Clone + PartialEq + Display + ExprOps,
{
    fn from(value: ExprType<T>) -> Self {
        let expr: Expr<T> = value.into();
        expr.into()
    }
}

impl<T> From<T> for ExprRef<T>
where
    T: Clone + PartialEq + Display + ExprOps,
{
    fn from(value: T) -> Self {
        let expr: ExprType<T> = value.into();
        expr.into()
    }
}

/* *****************************************************************************
 * Transform / From
 */

impl<T> ExprRef<T>
where
    T: Clone + PartialEq + Display + ExprOps + Debug,
{
    pub fn as_ref(&self) -> Ref<'_, Expr<T>> {
        self.0.as_ref().borrow()
    }

    pub fn resolve(&self) -> Result<()> {
        let mut expr = self.0.as_ref().borrow_mut();
        expr.resolve()?;
        Ok(())
    }

    pub fn eval(&self) -> Result<()> {
        self.resolve()?;
        let expr = self.as_ref();
        if let ExprType::Object(fields) = &expr.0 {
            fields.foreach(|_name, ex| ex.eval())?;
        }
        Ok(())
    }

    pub fn get_item(&self, name: &str) -> Result<ExprRef<T>> {
        self.resolve()?;
        match self.as_ref().get_item(name) {
            Some(item) => Ok(item),
            None => Err(Error::NoValue(format!("Invalid item '{}'", name))),
        }
    }
}

impl<T> Expr<T>
where
    T: Clone + PartialEq + Display + ExprOps + Debug,
{
    pub fn get_item(&self, item: &str) -> Option<ExprRef<T>> {
        match &self.0 {
            ExprType::Object(vars) => vars.get(item),
            _ => None,
        }
    }

    fn resolve_once(&mut self) -> Result<()> {
        let new_inner: Result<ExprType<T>> = match &self.0 {
            ExprType::BoundExpr(varspace, bound_expr) => match &bound_expr.as_ref().0 {
                ExprType::Object(fields) => {
                    Ok(ExprType::Object(fields.map(|val| {
                        ExprType::BoundExpr(varspace.clone(), val.clone()).into()
                    })))
                }
                ExprType::Let(fields, target_expr) => {
                    let mut vars: ExprSet<T> = varspace.clone();
                    for (field_name, field_expr) in fields {
                        let field_vars = vars.clone();
                        vars = vars.set(
                            field_name,
                            ExprType::BoundExpr(field_vars, field_expr.clone()).into(),
                        )?;
                    }
                    Ok(ExprType::BoundExpr(vars, target_expr.clone()))
                }
                ExprType::FuncDefIdent(arg_name, func_expr) => {
                    let new_scope = varspace.clone().unset(arg_name.as_str());
                    Ok(ExprType::FuncDefIdent(
                        arg_name.clone(),
                        ExprType::BoundExpr(new_scope, func_expr.clone()).into(),
                    ))
                }
                ExprType::FuncDefPattern(items, expr) => {
                    let mut new_scope = varspace.clone();
                    for item in items.iter() {
                        new_scope = new_scope.unset(item);
                    }
                    Ok(ExprType::FuncDefPattern(
                        items.clone(),
                        ExprType::BoundExpr(new_scope, expr.clone()).into(),
                    ))
                }
                ExprType::Var(name) => match &varspace.get(name) {
                    Some(value) => {
                        value.resolve()?;
                        Ok(value.as_ref().0.clone())
                    }
                    None => Err(Error::ScopeError(format!("Unknown variable {}", name))),
                },
                ExprType::UnOp(op, expr) => Ok(ExprType::UnOp(
                    *op,
                    ExprType::BoundExpr(varspace.clone(), expr.clone()).into(),
                )),
                ExprType::BinOp(op, lhs, rhs) => Ok(ExprType::BinOp(
                    *op,
                    ExprType::BoundExpr(varspace.clone(), lhs.clone()).into(),
                    ExprType::BoundExpr(varspace.clone(), rhs.clone()).into(),
                )),
                ExprType::FuncCall(func_name, arg_expr) => {
                    let funcexpr = varspace.get(func_name);
                    let bound_args = ExprType::BoundExpr(varspace.clone(), arg_expr.clone());
                    match funcexpr {
                        Some(func) => Ok(ExprType::CalledFunc(bound_args.into(), func.clone())),
                        None => Err(Error::ScopeError(format!(
                            "Unknown function name '{}'",
                            func_name
                        ))),
                    }
                }
                ExprType::Value(value) => Ok(ExprType::Value(value.clone())),
                ExprType::BoundExpr(_inner_vars, _inner_expr) => todo!(),
                ExprType::CalledFunc(_args, _func) => todo!(),
            },
            ExprType::CalledFunc(args_expr, func) => {
                args_expr.resolve()?;
                func.resolve()?;
                let funcref = func.as_ref();
                let (args, func_expr) = match &funcref.0 {
                    ExprType::FuncDefIdent(arg_name, func_expr) => {
                        Ok((ExprSet::single(arg_name, args_expr.clone()), func_expr))
                    }
                    ExprType::FuncDefPattern(arg_names, func_expr) => {
                        let mut new_vars = ExprSet::new();
                        for arg_name in arg_names {
                            let arg_value = args_expr.get_item(arg_name)?;
                            new_vars = new_vars.set(arg_name.clone(), arg_value)?;
                        }
                        Ok((new_vars, func_expr))
                    }
                    _ => Err(Error::ScopeError(format!(
                        "called func, but it's a {}",
                        func
                    ))),
                }?;

                // If function contains a bound scope, it should still apply,
                // and not overwrite input arguments.
                match &func_expr.as_ref().0 {
                    ExprType::BoundExpr(varspace, inner_expr) => Ok(ExprType::BoundExpr(
                        varspace.clone().merge(&args),
                        inner_expr.clone(),
                    )),
                    _ => Ok(ExprType::BoundExpr(args, func_expr.clone())),
                }
            }
            ExprType::UnOp(op, expr) => {
                expr.resolve()?;
                match op {
                    ExprUnOp::Neg => match &expr.as_ref().0 {
                        ExprType::Value(value) => Ok(ExprType::Value(value.op_neg()?)),
                        _ => Err(Error::EvalError(format!("negating non-value: {}", expr))),
                    },
                    ExprUnOp::Not => match &expr.as_ref().0 {
                        ExprType::Value(value) => Ok(ExprType::Value(value.op_not()?)),
                        _ => Err(Error::EvalError(format!("negating non-value: {}", expr))),
                    },
                }
            }
            ExprType::BinOp(op, lhs, rhs) => {
                lhs.resolve()?;
                match &lhs.as_ref().0 {
                    ExprType::Object(_lhs_obj) => todo!(),
                    ExprType::Value(lhs_val) => match op {
                        ExprBinOp::LogAnd => match lhs_val.as_bool()? {
                            true => {
                                rhs.resolve()?;
                                Ok(rhs.as_ref().0.clone())
                            }
                            false => Ok(ExprType::Value(lhs_val.new_from_bool(false))),
                        },
                        ExprBinOp::LogOr => match lhs_val.as_bool()? {
                            true => Ok(ExprType::Value(lhs_val.new_from_bool(true))),
                            false => {
                                rhs.resolve()?;
                                Ok(rhs.as_ref().0.clone())
                            }
                        },
                        ExprBinOp::LogImpl => match lhs_val.as_bool()? {
                            false => Ok(ExprType::Value(lhs_val.new_from_bool(true))),
                            true => {
                                rhs.resolve()?;
                                Ok(rhs.as_ref().0.clone())
                            }
                        },
                        _ => {
                            rhs.resolve()?;
                            match &rhs.as_ref().0 {
                                ExprType::Object(_rhs_obj) => todo!(),
                                ExprType::Value(rhs_val) => match op {
                                    ExprBinOp::AttrSel => todo!(),
                                    ExprBinOp::HasAttr => todo!(),
                                    ExprBinOp::ListConcat => todo!(),
                                    ExprBinOp::Mult => {
                                        Ok(ExprType::Value(T::op_mult(lhs_val, rhs_val)?))
                                    }
                                    ExprBinOp::Div => {
                                        Ok(ExprType::Value(T::op_div(lhs_val, rhs_val)?))
                                    }
                                    ExprBinOp::Sub => {
                                        Ok(ExprType::Value(T::op_sub(lhs_val, rhs_val)?))
                                    }
                                    ExprBinOp::Add => {
                                        Ok(ExprType::Value(T::op_add(lhs_val, rhs_val)?))
                                    }
                                    ExprBinOp::Update => todo!(),
                                    ExprBinOp::Lt => {
                                        Ok(ExprType::Value(T::op_lt(lhs_val, rhs_val)?))
                                    }
                                    ExprBinOp::Le => {
                                        Ok(ExprType::Value(T::op_le(lhs_val, rhs_val)?))
                                    }
                                    ExprBinOp::Gt => {
                                        Ok(ExprType::Value(T::op_gt(lhs_val, rhs_val)?))
                                    }
                                    ExprBinOp::Ge => {
                                        Ok(ExprType::Value(T::op_ge(lhs_val, rhs_val)?))
                                    }
                                    ExprBinOp::Eq => {
                                        Ok(ExprType::Value(T::op_eq(lhs_val, rhs_val)?))
                                    }
                                    ExprBinOp::Neq => {
                                        Ok(ExprType::Value(T::op_neq(lhs_val, rhs_val)?))
                                    }
                                    _ => unreachable!(),
                                },
                                typ => Err(Error::EvalError(format!(
                                    "Resolving unresolvable type {}",
                                    typ
                                ))),
                            }
                        }
                    },
                    typ => Err(Error::EvalError(format!(
                        "Resolving unresolvable type {}",
                        typ
                    ))),
                }
            }
            _ => unreachable!(),
        };
        self.0 = new_inner?;
        Ok(())
    }

    pub fn resolve(&mut self) -> Result<()> {
        while match self.0 {
            ExprType::Object(..) => false,
            ExprType::Value(..) => false,
            ExprType::Var(..) => true,
            ExprType::UnOp(..) => true,
            ExprType::BinOp(..) => true,
            ExprType::FuncDefIdent(..) => false,
            ExprType::FuncDefPattern(..) => false,
            ExprType::Let(..) => true,
            ExprType::FuncCall(..) => true,
            ExprType::BoundExpr(..) => true,
            ExprType::CalledFunc(..) => true,
        } {
            self.resolve_once()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_str;
    use crate::value::Value;

    fn eval(code: &str) -> ExprRef<Value> {
        let expr: ExprRef<Value> =
            ExprType::BoundExpr(ExprSet::new(), parse_str(code).unwrap()).into();
        expr.eval().unwrap();
        expr
    }

    #[test]
    fn test_resolve() -> Result<()> {
        let expr = parse_str(
            r#"
                {
                    stuff = "hello";
                    something = "hej";
                }
            "#,
        )
        .unwrap();
        let value = expr.get_item("stuff")?;
        assert_eq!(value, ExprRef::from(Value::String("hello".into())));
        Ok(())
    }

    #[test]
    fn test_resolve_deep() -> Result<()> {
        // This also tests "inner" as prefixed for reserved keyword "in" is ok
        let expr = parse_str(
            r#"
                {
                    stuff = "hello";
                    something = {
                        inner = 55;
                    };
                }
            "#,
        )
        .unwrap();
        let value = expr.get_item("something")?.get_item("inner")?;
        assert_eq!(value, ExprRef::from(Value::Int(55)));
        Ok(())
    }

    #[test]
    fn test_let() {
        let value = eval(
            r#"
                let
                    a = 12;
                    b = 75;
                in
                b
            "#,
        );
        assert_eq!(value, ExprRef::from(Value::Int(75)));
    }

    #[test]
    fn test_invalid_var() -> Result<()> {
        let expr: ExprRef<Value> =
            ExprType::BoundExpr(ExprSet::new(), parse_str("invalid_var").unwrap()).into();
        if let Err(Error::ScopeError(message)) = expr.resolve() {
            assert_eq!(message.as_str(), "Unknown variable invalid_var");
        } else {
            assert!(false);
        }
        Ok(())
    }

    #[test]
    fn test_let_set_var() {
        assert_eq! {
            eval(r#"
                let
                    a = 12;
                in
                {
                    stuff = a;
                }
            "#),
            eval("{ stuff = 12; }"),
        }
    }

    #[test]
    fn test_let_set_var_seq() {
        assert_eq! {
            eval(r#"
                let
                    a = 12;
                    b = a;
                in
                {
                    stuff = b;
                }
            "#),
            eval("{ stuff = 12; }"),
        }
    }

    #[test]
    fn test_func_call() {
        let func_a = parse_str("var: 13").unwrap();
        let func_b = parse_str("var: 42").unwrap();
        let call = parse_str("func_b 32").unwrap();
        let varscope =
            ExprSet::from(vec![("func_a", func_a.into()), ("func_b", func_b.into())]).unwrap();
        let value: ExprRef<Value> = ExprType::BoundExpr(varscope, call).into();
        value.resolve().unwrap();
        assert_eq!(value, ExprRef::from(Value::Int(42)));
    }

    #[test]
    fn test_func_call_var_arg() {
        let func_var = parse_str("var: var").unwrap();
        let arg_var = parse_str("32").unwrap();
        let call = parse_str("func arg").unwrap();
        let varscope =
            ExprSet::from(vec![("func", func_var.into()), ("arg", arg_var.into())]).unwrap();
        let value: ExprRef<Value> = ExprType::BoundExpr(varscope, call).into();
        value.resolve().unwrap();
        assert_eq!(value, ExprRef::from(Value::Int(32)));
    }

    #[test]
    fn test_func_call_resolved() {
        assert_eq! {
            eval(r#"
                let
                    a = 12;
                    func = test: {
                        var = test;
                    };
                in
                {
                    stuff = func a;
                }
            "#),
            eval("{ stuff = { var = 12; }; }"),
        }
    }

    #[test]
    fn test_func_call_bound() {
        assert_eq! {
            eval(r#"
                let
                    a = 12;
                    func = test: {
                        var = a;
                    };
                in
                {
                    stuff = func 77;
                }
            "#),
            eval("{ stuff = { var = 12; }; }"),
        }
    }

    #[test]
    fn test_func_call_resolved_stacked_let() {
        assert_eq! {
            eval(r#"
                let
                    a = 12;
                in
                let
                    func = test: {
                        var = test;
                    };
                in
                {
                    stuff = func a;
                }
            "#),
            eval("{ stuff = { var = 12; }; }"),
        }
    }

    #[test]
    fn test_func_call_pattern() {
        assert_eq! {
            eval(r#"
                let
                    a = 12;
                    b = 13;
                    func = { a, b, ... }: {
                        var = b;
                    };
                in
                {
                    stuff = func {
                        a = 15;
                        b = 74;
                    };
                }
            "#),
            eval("{ stuff = { var = 74; }; }"),
        }
    }

    #[test]
    fn test_eval() {
        assert_eq! {
            eval(r#"
                let
                    a = 12;
                    b = { inner = 43; };
                    myfunc = {target, ...}: { var = b; };
                in
                {
                    app = myfunc {
                        target = "app.elf";
                    };
                }
            "#),
            eval("{ app = { var = { inner = 43; }; }; }"),
        }
    }

    #[test]
    fn test_arith() {
        assert_eq! {
            eval("2 * 3 + 4 * 5"),
            eval("6 + 20"),
        }
        assert_eq! {
            eval("6 + 20"),
            eval("26"),
        }
    }

    #[test]
    fn test_bool_op() {
        assert_eq!(eval("false || 12"), eval("12"));
        assert_eq!(eval("true || 12"), eval("true"));
        assert_eq!(eval("false && 12"), eval("false"));
        assert_eq!(eval("true && 12"), eval("12"));
    }

    #[test]
    fn test_bool_laziness() {
        assert_eq!(eval("true || invalid_var"), eval("true"));
        assert_eq!(eval("false && invalid_var"), eval("false"));
        assert_eq!(eval("false -> invalid_var"), eval("true"));
    }

    #[test]
    fn test_bool_implication() {
        assert_eq!(eval("false -> false"), eval("true"));
        assert_eq!(eval("false -> true"), eval("true"));
        assert_eq!(eval("true -> false"), eval("false"));
        assert_eq!(eval("true -> true"), eval("true"));
        assert_eq!(eval("false -> 12"), eval("true"));
        assert_eq!(eval("true -> 12"), eval("12"));
    }

    #[test]
    fn test_bool_not() {
        assert_eq!(eval("!true"), eval("false"));
        assert_eq!(eval("!false"), eval("true"));
    }

    #[test]
    fn test_bool_neg() {
        assert_eq!(eval("let a = 5; in (-a) + 3"), eval("-2"));
    }

    #[test]
    fn test_func_call_laziness() {
        // The code contains an error; myfunc, which is not a function.
        // It is intentional that the func should not be evaluated, since
        // laziness in "false && ...", and therefore not be resolved as an
        // error.
        //
        // Test evalutes that eval is successful rather than ethe actual output
        assert_eq!(
            eval(
                r#"
                let
                    myfunc = not_a_function;
                    lazy_func_call = myfunc 72;
                in
                    false && lazy_func_call
                "#
            ),
            eval("false")
        );
    }
}
