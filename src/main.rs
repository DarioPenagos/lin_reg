mod automaton;
mod bool_alg;
mod parse;
use clap::Parser;
use std::{
    fs::read_to_string,
    io::{Error, ErrorKind},
    path::PathBuf,
};

use crate::automaton::Automaton;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File to scan
    #[arg(short, long)]
    file: PathBuf,

    /// Regex to consider
    #[arg(short, long)]
    regex: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let file = read_to_string(args.file)?;

    let automaton = Automaton::from_str(&args.regex)
        .ok_or(Error::new(ErrorKind::Other, "Passed invalid regex!"))?;

    for l in file.lines() {
        let mut matcher = automaton::Match::new(&automaton);
        if matcher.recognizes(l) {
            println!("{l}")
        }
    }
    Ok(())
}
