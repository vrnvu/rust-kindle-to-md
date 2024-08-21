use std::{
    collections::HashSet,
    fs::{read_to_string, File},
    io::{BufRead, BufReader},
};

use collection::{Collection, Quote};

pub mod collection;

fn read_lines(filename: &str) -> Vec<String> {
    let mut result = Vec::new();

    for line in read_to_string(filename).unwrap().lines() {
        result.push(line.to_string())
    }

    result
}

fn read_hashes_from_file(file_path: &str) -> anyhow::Result<HashSet<String>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut hashes = HashSet::new();
    for hash in reader.lines() {
        let hash = hash?;
        hashes.insert(hash);
    }

    Ok(hashes)
}

fn main() -> anyhow::Result<()> {
    let file_clippings = "My Clippings.txt";
    let lines = read_lines(file_clippings);

    let file_filters = "filters.txt";
    let filters = read_hashes_from_file(file_filters)?;

    let collection: Collection = lines
        .chunks(5)
        .flat_map(Quote::try_from)
        .filter(|quote| !filters.contains(quote.hash()))
        .collect();

    // Open the file in write mode, creating it if it doesn't exist
    let mut file_write = File::create("README.md")?;
    collection.write_quotes_to_file(&mut file_write)?;

    Ok(())
}
