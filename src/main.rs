pub(crate) mod datamodel;
pub mod error;
mod grammar;
pub(crate) mod immap;

use clap::Parser;
use error::Result;
use grammar::DnjParser;
use std::{path::PathBuf, process::exit};

use datamodel::Scope;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Root description file
    #[arg(short, long)]
    input: PathBuf,
}

fn run(args: Args) -> Result<()> {
    let expr = DnjParser::parse_file(args.input)?;
    let scope = Scope::new();
    println!("input: {:#}", expr);
    let resolved = scope.eval(expr).unwrap();
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
