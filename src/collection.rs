use std::{collections::HashMap, fmt::Display};

use anyhow::Context;
use regex::Regex;
use sha2::{Digest, Sha256};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Author(pub String);

impl Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for Author {
    fn from(value: &str) -> Self {
        Author(value.to_string())
    }
}

impl TryFrom<&String> for Author {
    type Error = anyhow::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let re_author = Regex::new(r"\(([\w ]+)\)$").unwrap();
        let author = re_author
            .captures(value)
            .with_context(|| format!("failed to find author in string: '{}'", value))?
            .get(1)
            .context("author was not found")?
            .as_str()
            .trim();
        Ok(author.into())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Book(pub String);

impl Display for Book {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for Book {
    fn from(value: &str) -> Self {
        Book(value.to_string())
    }
}

impl TryFrom<&String> for Book {
    type Error = anyhow::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let re_book = Regex::new(r"^[^()]+").unwrap();
        let book = re_book
            .find(value)
            .with_context(|| format!("failed to find book in string: '{}'", value))?
            .as_str()
            .trim();
        Ok(book.into())
    }
}

#[derive(Debug, Default)]
pub struct Quote {
    pub author: Author,
    pub book: Book,
    pub quote: String,
    pub hash: String,
}

#[derive(Debug, Default)]
pub struct Collection {
    collection: HashMap<Author, Vec<Quote>>,
}

impl Collection {
    fn new() -> Self {
        Collection {
            collection: HashMap::new(),
        }
    }

    pub fn authors(&self) -> Vec<&Author> {
        self.collection.keys().collect()
    }

    pub fn get(&self, author: &Author) -> Option<&Vec<Quote>> {
        self.collection.get(author)
    }

    fn add_quote(&mut self, quote: Quote) {
        self.collection
            .entry(quote.author.clone())
            .or_default()
            .push(quote);
    }
}

impl Extend<Quote> for Collection {
    fn extend<T: IntoIterator<Item = Quote>>(&mut self, iter: T) {
        for quote in iter {
            self.add_quote(quote);
        }
    }
}

impl FromIterator<Quote> for Collection {
    fn from_iter<T: IntoIterator<Item = Quote>>(iter: T) -> Self {
        let mut collection = Collection::new();
        collection.extend(iter);
        collection
    }
}

impl TryFrom<&[String]> for Quote {
    type Error = anyhow::Error;

    fn try_from(chunk: &[String]) -> Result<Self, Self::Error> {
        let author = Author::try_from(&chunk[0])?;
        let book = Book::try_from(&chunk[0])?;
        let quote = chunk[3].to_string();
        let hash = format!("{:x}", Sha256::digest(&quote));
        Ok(Quote {
            author,
            book,
            quote,
            hash,
        })
    }
}
