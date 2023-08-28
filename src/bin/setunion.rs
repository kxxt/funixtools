use clap::Parser;
use color_eyre::eyre::Result;
use itertools::join;
use std::{collections::BTreeSet, fs};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short, long, default_value_t=String::from("\n"))]
    separator: String,
    left: String,
    right: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    color_eyre::install()?;
    let left_input = fs::read_to_string(cli.left)?;
    let right_input = fs::read_to_string(cli.right)?;
    let left_set: BTreeSet<_> = left_input.split(&cli.separator).collect();
    let right_set: BTreeSet<_> = right_input.split(&cli.separator).collect();
    let union = left_set.union(&right_set);
    println!("{}", join(union, &cli.separator));
    Ok(())
}
