pub mod builtins;
pub mod context;
pub mod lang;
pub mod ninjawriter;
pub mod path;
pub mod value;

use clap::Parser;
use context::LangContext;
use lang::{Expr, Result};
use std::{path::PathBuf, process::exit};
use value::Value;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Root description file
    #[arg(short, long)]
    input: PathBuf,
}

fn run(args: Args) -> Result<()> {
    let mut ctx: LangContext = LangContext::new();
    let main_file = ctx.virtualize_path("root", &args.input)?;
    let expr: Expr<Value> = ctx.include(main_file)?;
    println!("input: {:#}", expr);
    expr.eval()?;
    println!("output: {:#}", expr);
    Ok(())
}

fn main() {
    match run(Args::parse()) {
        Ok(_) => {
            exit(0);
        }
        Err(err) => {
            println!("{}", err);
            exit(1);
        }
    }
}
