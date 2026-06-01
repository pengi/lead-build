use std::{fmt::Display, rc::Rc};

use crate::immap::ImMap;

pub type ExprSet = ImMap<Rc<Expr>>;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Object(ExprSet),
    Int(i64),
    String(String),
    Var(String),
    FuncDefIdent(String, Rc<Expr>),
    FuncDefPattern(Vec<String>, Rc<Expr>),
    Let(Vec<(String, Rc<Expr>)>, Rc<Expr>),
    FuncCall(String, Rc<Expr>),
    BoundExpr(ExprSet, Rc<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Object(im_map) => im_map.fmt(f),
            Expr::Int(val) => val.fmt(f),
            Expr::String(val) => write!(f, "{:?}", val),
            Expr::Var(val) => val.fmt(f),
            Expr::FuncDefIdent(name, expr) => write!(f, "{}: {}", name, expr),
            Expr::FuncDefPattern(items, expr) => {
                f.write_str("{")?;
                for item in items {
                    item.fmt(f)?;
                    f.write_str(", ")?;
                }
                f.write_str("...}: ")?;
                expr.fmt(f)?;
                Ok(())
            }
            Expr::Let(items, expr) => {
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
            Expr::FuncCall(name, expr) => write!(f, "{} {}", name, expr),
            Expr::BoundExpr(scope, expr) => write!(f, "[ {} @ {} ]", scope, expr),
        }
    }
}
