use std::{
    fmt::{Debug, Display},
    fs,
    path::PathBuf,
    rc::Rc,
};

use crate::lang::{
    Expr, ExprSet, ExprType, ParsableValue, Result,
    ops::{ExprBuiltin, ExprOps},
    parse_str,
};

#[derive(Debug)]
struct BuiltinInclude<T>(LangContext<T>, PathBuf)
where
    T: Clone + PartialEq + Display + ExprOps + ParsableValue + Debug;

impl<T> ExprBuiltin<T> for BuiltinInclude<T>
where
    T: Clone + PartialEq + Display + ExprOps + ParsableValue + Debug + 'static,
{
    fn get_name(&self) -> String {
        "include".into()
    }

    fn call(&self, arg: Expr<T>) -> crate::lang::ops::Result<Expr<T>> {
        let filename = arg.eval_string()?;
        let mut filepath = self.1.clone();
        filepath.push(filename);
        let result = self.0.read_file(filepath)?;
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct LangContext<T>
where
    T: Clone + PartialEq + Display + ExprOps + ParsableValue + Debug,
{
    builtins: Rc<ExprSet<T>>,
}

impl<T> Default for LangContext<T>
where
    T: Clone + PartialEq + Display + ExprOps + ParsableValue + Debug + 'static,
{
    fn default() -> Self {
        LangContext {
            builtins: ExprSet::default().into(),
        }
    }
}

impl<T> LangContext<T>
where
    T: Clone + PartialEq + Display + ExprOps + ParsableValue + Debug + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_builtin(&mut self, func: Rc<dyn ExprBuiltin<T>>) -> Result<()> {
        let builtin_name = func.get_name();
        let builtin_expr = Expr::new_builtin(func);
        self.builtins = self
            .builtins
            .as_ref()
            .clone()
            .set(builtin_name, builtin_expr)?
            .into();
        Ok(())
    }

    pub fn read_file(&self, filename: PathBuf) -> Result<Expr<T>> {
        let dirname: PathBuf = filename.parent().unwrap().into();
        let code = fs::read_to_string(filename).unwrap();
        let builtin_include = BuiltinInclude(self.clone(), dirname);
        let builtins = self.builtins.as_ref().clone().set(
            builtin_include.get_name(),
            Expr::new_builtin(Rc::new(builtin_include)),
        )?;
        let expr: Expr<T> = ExprType::BoundExpr(builtins, parse_str(&code)?).into();
        Ok(expr)
    }
}
