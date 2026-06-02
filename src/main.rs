pub mod error;
pub mod expr;
pub mod immap;
pub mod parser;
pub mod value;

use clap::Parser;
use error::Result;
use std::{path::PathBuf, process::exit};
use value::Value;

use crate::{
    expr::{Expr, ExprSet},
    parser::parse_file,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Root description file
    #[arg(short, long)]
    input: PathBuf,
}

fn run(args: Args) -> Result<()> {
    let expr: Expr<Value> = parse_file(args.input).unwrap().bind(ExprSet::new());
    println!("input: {:#}", expr);
    let resolved = expr.eval().unwrap();
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
