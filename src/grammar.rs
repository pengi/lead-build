use crate::error::Result;
use pest::{Parser, iterators::Pair};
use std::{collections::BTreeMap, fs, path::PathBuf};

mod parser {
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "grammar.pest"]
    pub struct Grammar;
}
pub type Error = pest::error::Error<parser::Rule>;

pub fn parse_file(path: PathBuf) -> Result<DnjExpr> {
    let content = fs::read_to_string(path)?;
    let parse_tree = parser::Grammar::parse(parser::Rule::entry, &content)?
        .next()
        .unwrap();
    let ast = DnjExpr::generate(parse_tree)?;
    Ok(ast)
}

#[derive(Debug)]
pub enum DnjExpr {
    Object { fields: BTreeMap<String, DnjExpr> },
    ConstInt { value: i64 },
    ConstFloat { value: f64 },
}

impl DnjExpr {
    fn generate(node: Pair<parser::Rule>) -> Result<DnjExpr> {
        assert!(node.as_rule() == parser::Rule::expr);

        let inner = node.into_inner().next().unwrap();
        match inner.as_rule() {
            parser::Rule::expr_object => Self::generate_object(inner),
            parser::Rule::const_int => Self::generate_const_int(inner),
            parser::Rule::const_float => Self::generate_const_float(inner),
            err_rule => panic!("Internal parse error, got {:?}", err_rule),
        }
    }

    fn generate_object(node: Pair<parser::Rule>) -> Result<DnjExpr> {
        let mut fields = BTreeMap::new();
        for assign in node.into_inner() {
            assert!(assign.as_rule() == parser::Rule::assignment);
            let mut inner = assign.into_inner();
            let ident = inner.next().unwrap().as_str().into();
            let value = DnjExpr::generate(inner.next().unwrap())?;
            fields.insert(ident, value);
        }
        Ok(DnjExpr::Object { fields })
    }

    fn generate_const_int(node: Pair<parser::Rule>) -> Result<DnjExpr> {
        Ok(DnjExpr::ConstInt {
            value: node.as_str().parse().expect("Internal parse int error"),
        })
    }

    fn generate_const_float(node: Pair<parser::Rule>) -> Result<DnjExpr> {
        Ok(DnjExpr::ConstFloat {
            value: node.as_str().parse().expect("Internal parse int error"),
        })
    }
}
