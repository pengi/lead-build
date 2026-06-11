use std::rc::Rc;

use crate::{
    lang::{Expr, ExprSet, ExprType, ops::ExprBuiltin},
    value::Value,
};

#[derive(Debug)]
struct VirtPathLockBuiltin;

impl ExprBuiltin<Value> for VirtPathLockBuiltin {
    fn get_name(&self) -> String {
        "lock".into()
    }

    fn call(
        &self,
        arg: crate::lang::Expr<Value>,
    ) -> crate::lang::ops::Result<crate::lang::Expr<Value>> {
        let val = arg.value()?;
        let path = val
            .try_as_path()
            .ok_or(crate::lang::ops::Error::Type(format!(
                "expected path, got {}",
                arg
            )))?;
        Ok(ExprType::Value(Value::Path(path.lock())).into())
    }
}

pub fn get_builtins() -> Expr<Value> {
    let path: Expr<Value> = Expr::from_builtins(vec![Rc::new(VirtPathLockBuiltin)]);

    ExprSet::from(vec![("_p", path)]).unwrap().into()
}
