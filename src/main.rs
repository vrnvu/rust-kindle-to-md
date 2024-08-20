use std::{
    collections::HashMap,
    fs::{read_to_string, File},
};

use std::io::Write;

use anyhow::Context;
use regex::Regex;
use sha2::{Digest, Sha256};

#[derive(Debug)]
struct Quote {
    author: String,
    book: String,
    quote: String,
    hash: Vec<u8>,
}

#[derive(Debug)]
struct Collection {
    collection: HashMap<String, Vec<Quote>>,
}

impl Collection {
    fn new() -> Self {
        Collection {
            collection: HashMap::new(),
        }
    }

    fn add_quote(&mut self, quote: Quote) {
        self.collection
            .entry(quote.author.clone())
            .or_insert_with(Vec::new)
            .push(quote);
    }
}

impl Quote {
    fn try_author(string: &String) -> anyhow::Result<String> {
        let re_author = Regex::new(r"\(([\w ]+)\)$").unwrap();
        let author = re_author
            .captures(string)
            .context("author was not found")?
            .get(1)
            .context("author was not found")?
            .as_str()
            .trim()
            .to_string();
        Ok(author)
    }

    fn try_book(string: &String) -> anyhow::Result<String> {
        let re_book = Regex::new(r"^[^()]+").unwrap();
        let book = re_book
            .find(string)
            .context("book was not found")?
            .as_str()
            .trim()
            .to_string();
        Ok(book)
    }
}

impl TryFrom<&[String]> for Quote {
    type Error = anyhow::Error;

    fn try_from(chunk: &[String]) -> Result<Self, Self::Error> {
        let author = Quote::try_author(&chunk[0])?;
        let book = Quote::try_book(&chunk[0])?;
        let quote = chunk[3].to_string();
        let hash = Sha256::digest(&quote).to_vec();
        Ok(Quote {
            author,
            book,
            quote,
            hash,
        })
    }
}

fn read_lines(filename: &str) -> Vec<String> {
    let mut result = Vec::new();

    for line in read_to_string(filename).unwrap().lines() {
        result.push(line.to_string())
    }

    result
}

fn authors(collection: &Collection) -> Vec<&String> {
    let mut authors: Vec<&String> = collection
        .collection
        .keys()
        .clone()
        .collect::<Vec<&String>>();

    authors.sort();
    authors
}

fn write_quotes_to_file(collection: &Collection) -> anyhow::Result<()> {
    // Open the file in write mode, creating it if it doesn't exist
    let mut file = File::create("README.md")?;

    let authors = authors(collection);
    for author in authors {
        // Write the author's name as a Markdown heading
        writeln!(file, "\n# {}\n", author)?;

        // Get the quotes for the author
        if let Some(quotes) = collection.collection.get(author) {
            // Group quotes by book title
            let mut quotes_by_book: HashMap<&str, Vec<&Quote>> = HashMap::new();
            for quote in quotes {
                quotes_by_book
                    .entry(&quote.book)
                    .or_insert_with(Vec::new)
                    .push(quote);
            }

            // Write each book title and its quotes
            for (book, book_quotes) in quotes_by_book {
                writeln!(file, "## {}\n", book)?;
                for quote in book_quotes {
                    writeln!(file, "> \"{}\"", quote.quote)?;
                }
                writeln!(file)?; // Add an extra newline for spacing
            }
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let filename = "My Clippings.txt";
    let lines = read_lines(filename);
    let quotes = lines
        .chunks(5)
        .flat_map(|c| Quote::try_from(c))
        .collect::<Vec<Quote>>();

    let mut collection = Collection::new();
    for quote in quotes {
        collection.add_quote(quote);
    }

    write_quotes_to_file(&collection)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_author_success() {
        let input = "Indigno de ser humano (Osamu Dazai)".to_string();
        let expected_author = "Osamu Dazai".to_string();

        let result = Quote::try_author(&input).expect("Author should be found");

        assert_eq!(result, expected_author);
    }

    #[test]
    fn test_try_author_success_special_char_in_name() {
        let input = "El idiota (Fiódor Dostoyevski)".to_string();
        let expected_author = "Fiódor Dostoyevski".to_string();

        let result = Quote::try_author(&input).expect("Author should be found");

        assert_eq!(result, expected_author);
    }

    #[test]
    fn test_try_author_failure() {
        let input = "Indigno de ser humano".to_string(); // No author provided
        let result = Quote::try_author(&input);

        assert!(
            result.is_err(),
            "Expected an error when author is not found"
        );
    }

    #[test]
    fn test_try_book_success() {
        let input = "Indigno de ser humano (Osamu Dazai)".to_string();
        let expected_book = "Indigno de ser humano".to_string();

        let result = Quote::try_book(&input).expect("Book should be found");

        assert_eq!(result, expected_book);
    }

    #[test]
    fn test_try_book_failure() {
        let input = "(Osamu Dazai)".to_string(); // No book title provided
        let result = Quote::try_book(&input);

        assert!(
            result.is_err(),
            "Expected an error when book title is not found"
        );
    }

    #[test]
    fn test_try_book_with_special_characters() {
        let input = "¡Indigno de ser humano! (Osamu Dazai)".to_string();
        let expected_book = "¡Indigno de ser humano!".to_string();

        let result = Quote::try_book(&input).expect("Book should be found");

        assert_eq!(result, expected_book);
    }

    #[test]
    fn test_try_author_with_extra_spaces() {
        let input = "Indigno de ser humano ( Osamu Dazai )".to_string();
        let expected_author = "Osamu Dazai".to_string();

        let result = Quote::try_author(&input).expect("Author should be found");

        assert_eq!(result, expected_author);
    }
}
