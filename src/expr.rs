use std::{fmt::Display, rc::Rc};

use crate::immap::ImMap;

pub type ExprSet = ImMap<Expr>;

#[derive(Debug, PartialEq)]
pub enum ExprType {
    Object(ExprSet),
    Int(i64),
    String(String),
    Var(String),
    FuncDefIdent(String, Expr),
    FuncDefPattern(Vec<String>, Expr),
    Let(Vec<(String, Expr)>, Expr),
    FuncCall(String, Expr),
    BoundExpr(ExprSet, Expr),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expr(pub Rc<ExprType>);

impl From<ExprType> for Expr {
    fn from(value: ExprType) -> Self {
        Expr(value.into())
    }
}

impl Display for ExprType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprType::Object(im_map) => im_map.fmt(f),
            ExprType::Int(val) => val.fmt(f),
            ExprType::String(val) => write!(f, "{:?}", val),
            ExprType::Var(val) => val.fmt(f),
            ExprType::FuncDefIdent(name, expr) => write!(f, "{}: {}", name, expr),
            ExprType::FuncDefPattern(items, expr) => {
                f.write_str("{")?;
                for item in items {
                    item.fmt(f)?;
                    f.write_str(", ")?;
                }
                f.write_str("...}: ")?;
                expr.fmt(f)?;
                Ok(())
            }
            ExprType::Let(items, expr) => {
                f.write_str("let ")?;
                for (var_name, var_expr) in items {
                    var_name.fmt(f)?;
                    f.write_str("=")?;
                    var_expr.fmt(f)?;
                    f.write_str("; ")?;
                }
                f.write_str("in ")?;
                expr.fmt(f)?;
                Ok(())
            }
            ExprType::FuncCall(name, expr) => write!(f, "{} {}", name, expr),
            ExprType::BoundExpr(scope, expr) => write!(f, "[ {} @ {} ]", scope, expr),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Expr {
    pub fn get_item(&self, item: &str) -> Option<Expr> {
        match self.0.as_ref() {
            ExprType::Object(vars) => vars.get(item),
            _ => None,
        }
    }
}
