use crate::{datamodel::Error::ScopeError, immap::ImMap};
use std::{fmt::Display, rc::Rc};

#[derive(Debug, PartialEq)]
pub enum Error {
    ScopeError(String, Scope),
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

#[derive(Debug, PartialEq)]
pub enum Expr {
    Object(ImMap<Rc<Expr>>),
    Int(i64),
    String(String),
    Var(String),
    FuncDefIdent(String, Rc<Expr>),
    FuncDefPattern(Vec<String>, Rc<Expr>),
    Let(Vec<(String, Rc<Expr>)>, Rc<Expr>),
    FuncCall(String, Rc<Expr>),
    BoundExpr(Scope, Rc<Expr>),
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

#[derive(Debug, Clone)]
pub struct Scope {
    vars: ImMap<Rc<Expr>>,
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.vars.fmt(f)
    }
}

impl PartialEq for Scope {
    // PartialEq for scope should never be called. It needs to be avaialble for
    // PartialEq for Expr to be availble, which is only needed for tests
    fn eq(&self, _other: &Self) -> bool {
        unimplemented!("PartialEq for Scope should not be called")
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ImMap<Rc<Expr>>> for Scope {
    fn from(vars: ImMap<Rc<Expr>>) -> Self {
        Self { vars }
    }
}

impl Scope {
    pub fn new() -> Scope {
        Scope { vars: ImMap::new() }
    }

    fn resolve_once(&self, expr: Rc<Expr>) -> Result<Rc<Expr>> {
        match expr.as_ref() {
            Expr::Let(fields, target_expr) => {
                let mut vars: ImMap<Rc<Expr>> = self.vars.clone();
                for (field_name, field_expr) in fields {
                    let var_scope: Scope = vars.clone().into();
                    vars =
                        vars.set(field_name.clone(), var_scope.bind(field_expr.clone()))?;
                }
                let var_scope: Scope = vars.into();
                Ok(var_scope.bind(target_expr.clone()))
            }
            Expr::BoundExpr(bound_scope, bound_expr) => match bound_expr.as_ref() {
                Expr::Object(im_map) => {
                    Ok(Expr::Object(im_map.map(|val| bound_scope.bind(val.clone()).into())).into())
                }
                Expr::FuncDefIdent(arg_name, func_expr) => {
                    let new_scope: Scope = bound_scope.vars.clone().unset(arg_name.as_str()).into();
                    Ok(Expr::FuncDefIdent(
                        arg_name.clone(),
                        new_scope.bind(func_expr.clone()).into(),
                    )
                    .into())
                }
                Expr::FuncDefPattern(items, expr) => {
                    let mut new_scope = bound_scope.clone();
                    for item in items {
                        new_scope.vars = new_scope.vars.unset(item);
                    }
                    Ok(
                        Expr::FuncDefPattern(items.clone(), new_scope.bind(expr.clone()).into())
                            .into(),
                    )
                }
                _ => bound_scope.resolve(bound_expr.clone()),
            },
            Expr::Var(name) => match self.vars.get(name) {
                Some(value) => Ok(value),
                None => Err(Error::ScopeError(
                    format!("Unknown variable {}", name),
                    self.clone(),
                )),
            },
            Expr::FuncCall(func_name, arg_expr) => match self.vars.get(func_name) {
                Some(func) => {
                    let func = self.resolve(func)?;
                    let (args, func_expr) = match func.as_ref() {
                        Expr::FuncDefIdent(arg_name, func_expr) => Ok((
                            ImMap::single(arg_name.clone(), self.bind(arg_expr.clone())),
                            func_expr,
                        )),
                        Expr::FuncDefPattern(arg_names, func_expr) => {
                            let mut new_vars = ImMap::new();
                            for arg_name in arg_names {
                                let arg_value = self.get_item(arg_expr.clone(), arg_name)?;
                                new_vars = new_vars.set(arg_name.clone(), arg_value)?;
                            }
                            Ok((new_vars, func_expr))
                        }
                        _ => Err(Error::ScopeError(
                            format!("called {}, which is a {}", func_name, func.to_string()),
                            self.clone(),
                        )),
                    }?;

                    // If function contains a bound scope, it should still apply,
                    // and not overwrite input arguments.
                    match func_expr.as_ref() {
                        Expr::BoundExpr(bound_scope, inner_expr) => {
                            let new_scope: Scope = bound_scope.vars.clone().merge(&args).into();
                            Ok(new_scope.bind(inner_expr.clone()))
                        }
                        _ => Ok(Expr::BoundExpr(args.into(), func_expr.clone()).into()),
                    }
                }
                None => Err(Error::ScopeError(
                    format!("Unknown function name '{}'", func_name),
                    self.clone(),
                )),
            },
            _ => Err(ScopeError(
                format!("Resolving invalid type {}", expr.to_string()),
                self.clone(),
            )),
        }
    }

    pub fn resolve(&self, expr: Rc<Expr>) -> Result<Rc<Expr>> {
        let mut expr = expr;
        while match expr.as_ref() {
            Expr::Object(..) => false,
            Expr::Int(..) => false,
            Expr::String(..) => false,
            Expr::Var(..) => true,
            Expr::FuncDefIdent(..) => false,
            Expr::FuncDefPattern(..) => false,
            Expr::Let(..) => true,
            Expr::FuncCall(..) => true,
            Expr::BoundExpr(..) => true,
        } {
            expr = self.resolve_once(expr)?;
        }
        Ok(expr)
    }

    pub fn eval(&self, expr: Rc<Expr>) -> Result<Rc<Expr>> {
        let res = self.resolve(expr)?;
        match res.as_ref() {
            Expr::Object(im_map) => {
                Ok(Expr::Object(im_map.map(|e| self.eval(e.clone()).unwrap())).into())
            }
            _ => Ok(res),
        }
    }

    fn bind(&self, expr: Rc<Expr>) -> Rc<Expr> {
        Expr::BoundExpr(self.clone(), expr).into()
    }

    pub fn get_item(&self, expr: Rc<Expr>, item: &str) -> Result<Rc<Expr>> {
        let expr = self.resolve(expr)?;
        let out = match expr.as_ref() {
            Expr::Object(fields) => {
                let field = fields
                    .get(item)
                    .ok_or_else(|| Error::ScopeError("field not found".into(), self.clone()))?;
                Ok(field.clone())
            }
            _ => Err(Error::ScopeError("get_item resolving".into(), self.clone())),
        }?;
        self.resolve(out)
    }

    // Currently only used for testing
    #[cfg(test)]
    pub fn get_path<'a>(
        &self,
        expr: Rc<Expr>,
        path: impl Iterator<Item = &'a str>,
    ) -> Result<Rc<Expr>> {
        let mut cur = expr;
        for item in path {
            cur = self.resolve(cur)?;
            cur = match cur.as_ref() {
                Expr::Object(fields) => {
                    let field = fields.get(item).ok_or_else(|| {
                        Error::ScopeError(format!("field {} not found", item), self.clone())
                    })?;
                    Ok(field.clone())
                }
                _ => Err(Error::ScopeError("get_path resolving".into(), self.clone())),
            }?;
        }
        cur = self.resolve(cur)?;
        Ok(cur)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::DnjParser;

    macro_rules! assert_dnj_value {
        ($code:expr, $path:expr, $value:expr) => {
            let expr = DnjParser::parse_str($code).unwrap();
            let scope = Scope::default();
            let value = scope.get_path(expr, $path.into_iter()).unwrap();
            assert_eq!(value, $value.into());
        };
    }

    fn eval(code: &str) -> Rc<Expr> {
        Scope::new()
            .eval(DnjParser::parse_str(code).unwrap())
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
        let scope = Scope::default();
        let value = scope.get_item(expr, "stuff").unwrap();
        assert_eq!(*value, Expr::String("hello".into()));
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
        let scope = Scope::default();
        let value = scope
            .get_path(expr, vec!["something", "inner"].into_iter())
            .unwrap();
        assert_eq!(*value, Expr::String("deep".into()));
    }

    #[test]
    fn test_let() {
        let expr = DnjParser::parse_str(
            r#"
                let
                    a = 12;
                    b = "hello";
                in
                b
            "#,
        )
        .unwrap();
        let scope = Scope::default();
        let value = scope.resolve(expr).unwrap();
        assert_eq!(*value, Expr::String("hello".into()));
    }

    #[test]
    fn test_invalid_var() {
        let expr = DnjParser::parse_str(
            r#"
                invalid_var
            "#,
        )
        .unwrap();
        let scope = Scope::default();
        if let Error::ScopeError(message, _) = scope.resolve(expr).unwrap_err() {
            assert_eq!(message.as_str(), "Unknown variable invalid_var");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_let_set_var() {
        assert_dnj_value! {
            r#"
                let
                    a = 12;
                in
                {
                    stuff = a;
                }
            "#,
            vec!["stuff"],
            Expr::Int(12)
        }
    }

    #[test]
    fn test_let_set_var_seq() {
        assert_dnj_value! {
            r#"
                let
                    a = 12;
                    b = a;
                in
                {
                    stuff = b;
                }
            "#,
            vec!["stuff"],
            Expr::Int(12)
        }
    }

    #[test]
    fn test_func_call() {
        let func_a = DnjParser::parse_str("var: 13").unwrap();
        let func_b = DnjParser::parse_str("var: 42").unwrap();
        let call = DnjParser::parse_str("func_b 32").unwrap();
        let scope: Scope =
            ImMap::from(vec![("func_a".into(), func_a), ("func_b".into(), func_b)].into_iter())
                .unwrap()
                .into();
        let value = scope.resolve(call).unwrap();
        assert_eq!(*value, Expr::Int(42));
    }

    #[test]
    fn test_func_call_var_arg() {
        let func_var = DnjParser::parse_str("var: var").unwrap();
        let arg_var = DnjParser::parse_str("32").unwrap();
        let call = DnjParser::parse_str("func arg").unwrap();
        let scope: Scope =
            ImMap::from(vec![("func".into(), func_var), ("arg".into(), arg_var)].into_iter())
                .unwrap()
                .into();
        let value = scope.resolve(call).unwrap();
        assert_eq!(*value, Expr::Int(32));
    }

    #[test]
    fn test_func_call_resolved() {
        assert_dnj_value! {
            r#"
                let
                    a = 12;
                    func = test: {
                        var = test;
                    };
                in
                {
                    stuff = func a;
                }
            "#,
            vec!["stuff", "var"],
            Expr::Int(12)
        }
    }

    #[test]
    fn test_func_call_bound() {
        assert_dnj_value! {
            r#"
                let
                    a = 12;
                    func = test: {
                        var = a;
                    };
                in
                {
                    stuff = func 77;
                }
            "#,
            vec!["stuff", "var"],
            Expr::Int(12)
        }
    }

    #[test]
    fn test_func_call_resolved_stacked_let() {
        assert_dnj_value! {
            r#"
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
            "#,
            vec!["stuff", "var"],
            Expr::Int(12)
        }
    }

    #[test]
    fn test_func_call_pattern() {
        assert_dnj_value! {
            r#"
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
            "#,
            vec!["stuff", "var"],
            Expr::Int(74)
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
