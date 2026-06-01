pub mod datamodel;
pub mod error;
pub mod expr;
mod grammar;
pub mod immap;

use clap::Parser;
use error::Result;
use grammar::DnjParser;
use std::{path::PathBuf, process::exit};

use datamodel::Scope;

use crate::expr::{Expr, ExprSet, ExprType};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Root description file
    #[arg(short, long)]
    input: PathBuf,
}

fn run(args: Args) -> Result<()> {
    let expr: Expr = DnjParser::parse_file(args.input)?;
    let wrapped: Expr = ExprType::BoundExpr(ExprSet::new(), expr).into();
    let scope = Scope::new();
    println!("input: {:#}", wrapped);
    let resolved = scope.eval(wrapped).unwrap();
    println!("output: {:#}", resolved);
    Ok(())
}

fn main() {
    match run(Args::parse()) {
        Ok(_) => {
            exit(0);
        }
        Err(err) => {
            println!("{}", err);
            println!("{:#?}", err);
            exit(1);
        }
    }
}
