use crate::expr::{Expr, ExprSet, ExprType};
use pest::{Span, error::ErrorVariant};
use pest_consume::{Parser, match_nodes};
use std::{fmt::Display, fs, path::PathBuf};

pub trait ParsableValue
where
    Self: Sized,
{
    fn parse_int(value: impl ToString) -> Option<Self>;
    fn parse_string(value: impl ToString) -> Option<Self>;
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct DnjParser;

pub type Error = pest_consume::Error<Rule>;
pub type Result<T> = std::result::Result<T, Error>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

impl DnjParser {
    pub fn parse_file<T>(path: PathBuf) -> Result<Expr<T>>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        let input_str = fs::read_to_string(path).unwrap();
        Self::parse_str(&input_str)
    }
    pub fn parse_str<T>(input_str: &str) -> Result<Expr<T>>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        let parse_tree = DnjParser::parse(Rule::entry, input_str)?;
        let input = parse_tree.single()?;
        DnjParser::entry(input)
    }
}

#[pest_consume::parser]
impl DnjParser {
    fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }

    fn entry<T>(input: Node) -> Result<Expr<T>>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        Ok(match_nodes! {input.into_children();
            [expr(e), EOI(_)] => e,
        })
    }

    /*
     * Expression
     */

    fn expr<T>(input: Node) -> Result<Expr<T>>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        Ok(match_nodes! {input.into_children();
            [object(x)] => x,
            [const_int(x)] => x,
            [const_str(x)] => x,
            [expr_func_call(x)] => x,
            [func_def(x)] => x,
            [let_def(x)] => x,
            [variable(x)] => x,
        })
    }

    /*
     * Primitives
     */

    fn object<T>(input: Node) -> Result<Expr<T>>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        let assignments = match_nodes! {input.into_children();
            [object_assignment(a)..] => a
        };
        let mut map: ExprSet<T> = ExprSet::new();
        for (key, value, span) in assignments {
            map = map.set(key, value).map_err(|err| {
                Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: err.to_string(),
                    },
                    span,
                )
            })?;
        }
        Ok(ExprType::Object(map).into())
    }

    fn object_assignment<T>(input: Node) -> Result<(String, Expr<T>, Span)>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        let span = input.as_span();
        Ok(match_nodes! {input.into_children();
            [ident(ident), expr(val)] => (ident, val, span),
        })
    }

    fn expr_func_call<T>(input: Node) -> Result<Expr<T>>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        Ok(match_nodes! {input.into_children();
            [ident(ident), expr(val)] => ExprType::FuncCall(ident, val).into(),
        })
    }

    fn variable<T>(input: Node) -> Result<Expr<T>>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        Ok(match_nodes! {input.into_children();
            [ident(ident)] => ExprType::Var(ident).into(),
        })
    }

    /*
     * Function definition
     */

    fn func_def<T>(input: Node) -> Result<Expr<T>>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        Ok(match_nodes! {input.into_children();
            [ident(ident), expr(val)] => ExprType::FuncDefIdent(ident, val.into()).into(),
            [func_args_pattern(pat), expr(val)] => ExprType::FuncDefPattern(pat, val.into()).into(),
        })
    }

    fn func_args_pattern(input: Node) -> Result<Vec<String>> {
        Ok(match_nodes! {input.into_children();
            [ident(ident)..] => ident,
        }
        .collect())
    }

    /*
     * Let blocks
     */

    fn let_def<T>(input: Node) -> Result<Expr<T>>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        let (bl, ex) = match_nodes! {input.into_children();
            [let_block(bl), expr(ex)] => (bl, ex)
        };
        Ok(ExprType::Let(bl, ex.into()).into())
    }

    fn let_block<T>(input: Node) -> Result<Vec<(String, Expr<T>)>>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        Ok(match_nodes! {input.into_children();
            [let_stmt(stmt)..] => stmt,
        }
        .collect())
    }

    fn let_stmt<T>(input: Node) -> Result<(String, Expr<T>)>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        Ok(match_nodes! {input.into_children();
            [ident(ident), expr(val)] => (ident, val.into()),
        })
    }

    /*
     * Literals
     */

    fn ident(input: Node) -> Result<String> {
        Ok(input.as_str().into())
    }

    fn const_int<T>(input: Node) -> Result<Expr<T>>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        let parse_res = T::parse_int(input.as_str());
        match parse_res {
            Some(value) => Ok(ExprType::Value(value).into()),
            None => Err(input.error("Unable to parse integer")),
        }
    }

    fn const_str<T>(input: Node) -> Result<Expr<T>>
    where
        T: ParsableValue + Clone + PartialEq + Display,
    {
        let err = input.error("Unable to parse string");
        let str_data: String = match_nodes! {input.into_children();
            [const_str_sym(c)..] => c,
        }
        .collect();
        let parse_res = T::parse_string(str_data);
        match parse_res {
            Some(value) => Ok(ExprType::Value(value).into()),
            None => Err(err),
        }
    }

    fn const_str_sym(input: Node) -> Result<char> {
        Ok(match_nodes! {input.into_children();
            [const_str_char(c)] => c,
            [const_str_esc(c)] => c,
            [const_str_hex(c)] => c,
        })
    }

    fn const_str_char(input: Node) -> Result<char> {
        Ok(input.as_str().chars().next().unwrap())
    }

    fn const_str_esc(input: Node) -> Result<char> {
        let chr = input.as_str().chars().next().unwrap();
        Ok(match chr {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            c => c,
        })
    }

    fn const_str_hex(input: Node) -> Result<char> {
        let str = input.as_str();
        let val = u32::from_str_radix(str, 16).unwrap();
        Ok(char::from_u32(val).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Clone, Debug)]
    enum TestValue {
        Int(i64),
        String(String),
    }

    impl Display for TestValue {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestValue::Int(v) => v.fmt(f),
                TestValue::String(v) => v.fmt(f),
            }
        }
    }

    impl ParsableValue for TestValue {
        fn parse_int(value: impl ToString) -> Option<Self> {
            Some(TestValue::Int(value.to_string().parse().unwrap()))
        }

        fn parse_string(value: impl ToString) -> Option<Self> {
            Some(TestValue::String(value.to_string()))
        }
    }

    #[test]
    fn test_parse_int() {
        let tree: Expr<TestValue> = DnjParser::parse_str("1231").unwrap();
        assert_eq!(Expr::from(ExprType::Value(TestValue::Int(1231))), tree);
    }

    #[test]
    fn test_parse_obj() {
        let code = r#"
            {
                boll = 123;
                hej = 323;
            }
        "#;
        let tree = DnjParser::parse_str(code).unwrap();
        assert_eq!(
            Expr::from(ExprType::Object(
                ExprSet::from([
                    ("boll", ExprType::Value(TestValue::Int(123)).into()),
                    ("hej", ExprType::Value(TestValue::Int(323)).into())
                ])
                .unwrap()
            )),
            tree
        );
    }

    #[test]
    fn test_parse_obj_in_obj() {
        let code = r#"
            {
                boll = 123;
                hej = { a=2; b=3; };
            }
        "#;
        let tree = DnjParser::parse_str(code).unwrap();
        assert_eq!(
            Expr::from(ExprType::Object(
                ExprSet::from([
                    ("boll", ExprType::Value(TestValue::Int(123)).into()),
                    (
                        "hej".into(),
                        ExprType::Object(
                            ExprSet::from([
                                ("a", ExprType::Value(TestValue::Int(2)).into()),
                                ("b", ExprType::Value(TestValue::Int(3)).into()),
                            ])
                            .unwrap()
                        )
                        .into()
                    )
                ])
                .unwrap()
            )),
            tree
        );
    }

    #[test]
    fn test_parse_str() {
        let code = "\"boll\\\"hej\\u0041\"";
        let tree = DnjParser::parse_str(code).unwrap();
        assert_eq!(
            Expr::from(ExprType::Value(TestValue::String("boll\"hejA".into()))),
            tree
        );
    }

    #[test]
    fn test_parse_func_call() {
        let code = "hej 12";
        let tree = DnjParser::parse_str(code).unwrap();
        assert_eq!(
            Expr::from(ExprType::FuncCall(
                "hej".into(),
                ExprType::Value(TestValue::Int(12)).into()
            )),
            tree
        );
    }

    #[test]
    fn test_parse_func_def_ident() {
        let code = "hej: 12";
        let tree = DnjParser::parse_str(code).unwrap();
        assert_eq!(
            Expr::from(ExprType::FuncDefIdent(
                "hej".into(),
                ExprType::Value(TestValue::Int(12)).into()
            )),
            tree
        );
    }

    #[test]
    fn test_parse_func_def_pattern_variadic() {
        let code = "{ hej, hopp, svej, ... }: 12";
        let tree = DnjParser::parse_str(code).unwrap();
        assert_eq!(
            Expr::from(ExprType::FuncDefPattern(
                vec!["hej".into(), "hopp".into(), "svej".into()],
                ExprType::Value(TestValue::Int(12)).into()
            )),
            tree
        );
    }

    #[test]
    fn test_parse_func_def_pattern_non_var_1() {
        let code = "{ hej, hopp, svej }: 12";

        let res: Result<Expr<TestValue>> = DnjParser::parse_str(code);
        // Should be an error, try to unwrap it. Panic otherwise
        let _ = res.unwrap_err();
    }

    #[test]
    fn test_parse_func_def_pattern_non_var_2() {
        let code = "{ hej, hopp, svej, }: 12";

        let res: Result<Expr<TestValue>> = DnjParser::parse_str(code);
        // Should be an error, try to unwrap it. Panic otherwise
        let _ = res.unwrap_err();
    }

    #[test]
    fn test_parse_let() {
        let code = "let a = 21; b = 33; in 434";
        let tree = DnjParser::parse_str(code).unwrap();
        assert_eq!(
            Expr::from(ExprType::Let(
                vec![
                    ("a".into(), ExprType::Value(TestValue::Int(21)).into()),
                    ("b".into(), ExprType::Value(TestValue::Int(33)).into()),
                ],
                ExprType::Value(TestValue::Int(434)).into(),
            )),
            tree
        );
    }
}
