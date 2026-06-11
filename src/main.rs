pub mod lang;
pub mod ninjawriter;
pub mod value;

use lang::{Expr, LangContext, Result};
use std::process::exit;
use value::Value;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Root description file
    #[arg(short, long)]
    input: PathBuf,
}

fn run(args: Args) -> Result<()> {
    let ctx: LangContext<Value> = LangContext::new();
    let expr: Expr<Value> = ctx.read_file(args.input)?;
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
