use clap::Parser;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use anyhow::{Context, Result};

/// Search for a pattern in a file to display the lines that contain it
#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    /// #[clap(short, long)]
    pattern: String,
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let path = &args.path;
    let f = File::open(path)
        .with_context(|| format!("could not real file`{:?}`", path.to_str()))?;
    let mut reader = BufReader::new(f);


    // let content = std::fs::read_to_string(&args.path)
    //     .expect("could not read file");

    for line in reader.lines() {
        if (line.as_ref().expect("line")).contains(&args.pattern) {
            println!("{}", line.expect("something"));
        }
        
    }

    // for line in content.lines() {
    //     if line.contains(&args.pattern) {
    //         println!("{}", line);
    //     }
    // }
    Ok(())
}
