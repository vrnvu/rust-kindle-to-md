use std::{
    collections::HashSet,
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::Context;
use collection::{Collection, Quote};

pub mod collection;

fn read_lines<P>(path: P) -> anyhow::Result<Vec<String>>
where
    P: AsRef<Path> + Debug,
{
    let file = File::open(&path).with_context(|| format!("Failed to open file at {:?}", &path))?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<String>, _>>()
        .with_context(|| format!("Failed to read lines from file at {:?}", path))?;

    Ok(lines)
}

fn read_hashes_from_file<P>(path: P) -> anyhow::Result<HashSet<String>>
where
    P: AsRef<Path> + Debug,
{
    let file = File::open(&path).with_context(|| format!("Failed to open file at {:?}", &path))?;
    let reader = BufReader::new(file);
    let hashes: HashSet<String> = reader
        .lines()
        .collect::<Result<HashSet<_>, _>>()
        .with_context(|| format!("Failed to read lines from file at {:?}", path))?;
    Ok(hashes)
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        panic!("cannot accept more than one argument")
    }
    let hash_present = if args.len() == 2 {
        match args[1].as_str() {
            "hash" => true,
            _ => panic!("Invalid argument! Only 'hash' is accepted."),
        }
    } else {
        false
    };

    let file_clippings = "My Clippings.txt";
    let lines = read_lines(file_clippings)?;

    let file_filters = "filters.txt";
    let filters = read_hashes_from_file(file_filters)?;

    let collection: Collection = lines
        .chunks(5)
        .flat_map(Quote::try_from)
        .filter(|quote| !filters.contains(quote.hash()))
        .collect();

    // Open the file in write mode, creating it if it doesn't exist
    let mut file_write = File::create("README.md")?;
    if hash_present {
        collection.write_quotes_with_hash_to_file(&mut file_write)?;
    } else {
        collection.write_quotes_to_file(&mut file_write)?;
    }

    Ok(())
}
