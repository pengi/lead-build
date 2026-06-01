use std::{fmt::Display, rc::Rc};

use crate::immap::ImMap;

/*
 * Error
 */

#[derive(Debug, PartialEq)]
pub enum Error {
    ScopeError(String, ExprSet),
    EvalError(String),
    DupKey(String),
}

impl From<crate::immap::Error> for Error {
    fn from(value: crate::immap::Error) -> Self {
        match value {
            crate::immap::Error::DupKey(key) => Error::DupKey(key),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

/*
 * Types
 */

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
pub struct Expr(Rc<ExprType>);

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

    fn resolve_once(&self) -> Result<Expr> {
        match self.0.as_ref() {
            ExprType::BoundExpr(varspace, bound_expr) => match bound_expr.0.as_ref() {
                ExprType::Object(fields) => Ok(ExprType::Object(
                    fields.map(|val| ExprType::BoundExpr(varspace.clone(), val.clone()).into()),
                )
                .into()),
                ExprType::Let(fields, target_expr) => {
                    let mut vars: ExprSet = varspace.clone();
                    for (field_name, field_expr) in fields {
                        let field_vars = vars.clone();
                        vars = vars.set(
                            field_name.clone(),
                            ExprType::BoundExpr(field_vars, field_expr.clone()).into(),
                        )?;
                    }
                    Ok(ExprType::BoundExpr(vars, target_expr.clone()).into())
                }
                ExprType::FuncDefIdent(arg_name, func_expr) => {
                    let new_scope = varspace.clone().unset(arg_name.as_str());
                    Ok(ExprType::FuncDefIdent(
                        arg_name.clone(),
                        ExprType::BoundExpr(new_scope, func_expr.clone()).into(),
                    )
                    .into())
                }
                ExprType::FuncDefPattern(items, expr) => {
                    let mut new_scope = varspace.clone();
                    for item in items {
                        new_scope = new_scope.unset(item);
                    }
                    Ok(ExprType::FuncDefPattern(
                        items.clone(),
                        ExprType::BoundExpr(new_scope, expr.clone()).into(),
                    )
                    .into())
                }
                ExprType::Var(name) => match varspace.get(name) {
                    Some(value) => Ok(value),
                    None => Err(Error::ScopeError(
                        format!("Unknown variable {}", name),
                        varspace.clone(),
                    )),
                },
                ExprType::FuncCall(func_name, arg_expr) => match varspace.get(func_name) {
                    Some(func) => {
                        let func = func.resolve()?; // TODO: wrong scope
                        let (args, func_expr) = match func.0.as_ref() {
                            ExprType::FuncDefIdent(arg_name, func_expr) => Ok((
                                ExprSet::single(
                                    arg_name.clone(),
                                    ExprType::BoundExpr(varspace.clone(), arg_expr.clone()).into(),
                                ),
                                func_expr,
                            )),
                            ExprType::FuncDefPattern(arg_names, func_expr) => {
                                let arg_expr = arg_expr.resolve()?;

                                let mut new_vars = ExprSet::new();
                                for arg_name in arg_names {
                                    let arg_value = match arg_expr.get_item(arg_name) {
                                        Some(x) => Ok(x),
                                        None => Err(Error::ScopeError(
                                            format!(
                                                "called {}, no attr {} found",
                                                func_name, arg_name
                                            ),
                                            varspace.clone(),
                                        )),
                                    }?;
                                    new_vars = new_vars.set(arg_name.clone(), arg_value)?;
                                }
                                Ok((new_vars, func_expr))
                            }
                            _ => Err(Error::ScopeError(
                                format!("called {}, which is a {}", func_name, func.to_string()),
                                varspace.clone(),
                            )),
                        }?;

                        // If function contains a bound scope, it should still apply,
                        // and not overwrite input arguments.
                        match func_expr.0.as_ref() {
                            ExprType::BoundExpr(varspace, inner_expr) => Ok(ExprType::BoundExpr(
                                varspace.clone().merge(&args),
                                inner_expr.clone(),
                            )
                            .into()),
                            _ => Ok(ExprType::BoundExpr(args.into(), func_expr.clone()).into()),
                        }
                    }
                    None => Err(Error::ScopeError(
                        format!("Unknown function name '{}'", func_name),
                        varspace.clone(),
                    )),
                },
                ExprType::Int(..) => Ok(bound_expr.clone()),
                ExprType::String(..) => Ok(bound_expr.clone()),
                ExprType::BoundExpr(_inner_vars, _expr) => todo!(),
            },
            _ => Err(Error::EvalError(format!(
                "Resolving unresolvable type {}",
                self.to_string()
            ))),
        }
    }

    pub fn resolve(&self) -> Result<Expr> {
        let mut expr: Expr = self.clone();
        while match expr.0.as_ref() {
            ExprType::Object(..) => false,
            ExprType::Int(..) => false,
            ExprType::String(..) => false,
            ExprType::Var(..) => true,
            ExprType::FuncDefIdent(..) => false,
            ExprType::FuncDefPattern(..) => false,
            ExprType::Let(..) => true,
            ExprType::FuncCall(..) => true,
            ExprType::BoundExpr(..) => true,
        } {
            expr = expr.resolve_once()?;
        }
        Ok(expr)
    }

    pub fn eval(&self) -> Result<Expr> {
        let res = self.resolve()?;
        match res.0.as_ref() {
            ExprType::Object(im_map) => {
                Ok(ExprType::Object(im_map.map(|e| e.eval().unwrap())).into())
            }
            _ => Ok(res),
        }
    }

    pub fn bind(self, vars: ExprSet) -> Expr {
        ExprType::BoundExpr(vars, self).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::DnjParser;

    fn eval(code: &str) -> Expr {
        DnjParser::parse_str(code)
            .unwrap()
            .bind(ExprSet::new())
            .eval()
            .unwrap()
    }

    #[test]
    fn test_resolve() {
        let expr = DnjParser::parse_str(
            r#"
                {
                    stuff = "hello";
                    something = "hej";
                }
            "#,
        )
        .unwrap();
        let value = expr.get_item("stuff").unwrap();
        assert_eq!(*value.0, ExprType::String("hello".into()));
    }

    #[test]
    fn test_resolve_deep() {
        // This also tests "inner" as prefixed for reserved keyword "in" is ok
        let expr = DnjParser::parse_str(
            r#"
                {
                    stuff = "hello";
                    something = {
                        inner = "deep";
                    };
                }
            "#,
        )
        .unwrap();
        let value = expr
            .get_item("something")
            .unwrap()
            .get_item("inner")
            .unwrap();
        assert_eq!(*value.0, ExprType::String("deep".into()));
    }

    #[test]
    fn test_let() {
        let value = DnjParser::parse_str(
            r#"
                let
                    a = 12;
                    b = "hello";
                in
                b
            "#,
        )
        .unwrap()
        .bind(ExprSet::new())
        .resolve()
        .unwrap();
        assert_eq!(*value.0, ExprType::String("hello".into()));
    }

    #[test]
    fn test_invalid_var() {
        let expr = DnjParser::parse_str(
            r#"
                invalid_var
            "#,
        )
        .unwrap()
        .bind(ExprSet::new());
        if let Error::ScopeError(message, _) = expr.resolve().unwrap_err() {
            assert_eq!(message.as_str(), "Unknown variable invalid_var");
        } else {
            assert!(false);
        }
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
        let func_a = DnjParser::parse_str("var: 13").unwrap();
        let func_b = DnjParser::parse_str("var: 42").unwrap();
        let call = DnjParser::parse_str("func_b 32").unwrap();
        let varscope =
            ExprSet::from(vec![("func_a".into(), func_a), ("func_b".into(), func_b)].into_iter())
                .unwrap();
        let value: Expr = call.bind(varscope).resolve().unwrap();
        assert_eq!(*value.0, ExprType::Int(42));
    }

    #[test]
    fn test_func_call_var_arg() {
        let func_var = DnjParser::parse_str("var: var").unwrap();
        let arg_var = DnjParser::parse_str("32").unwrap();
        let call = DnjParser::parse_str("func arg").unwrap();
        let varscope =
            ExprSet::from(vec![("func".into(), func_var), ("arg".into(), arg_var)].into_iter())
                .unwrap();
        let value: Expr = call.bind(varscope).resolve().unwrap();
        assert_eq!(*value.0, ExprType::Int(32));
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
}
