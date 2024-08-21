use std::{collections::BTreeMap, fmt::Display};

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
    author: Author,
    book: Book,
    quote: String,
    hash: String,
}

impl Quote {
    pub fn hash(&self) -> &String {
        &self.hash
    }
}

#[derive(Debug, Default)]
pub struct Collection {
    collection: BTreeMap<Author, Vec<Quote>>,
}

impl Collection {
    fn new() -> Self {
        Collection {
            collection: BTreeMap::new(),
        }
    }

    fn authors(&self) -> Vec<&Author> {
        self.collection.keys().collect()
    }

    fn get(&self, author: &Author) -> Option<&Vec<Quote>> {
        self.collection.get(author)
    }

    fn add_quote(&mut self, quote: Quote) {
        self.collection
            .entry(quote.author.clone())
            .or_default()
            .push(quote);
    }

    pub fn write_quotes_with_hash_to_file(
        self,
        file: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        let authors = self.authors();

        // Write the index at the top of the file
        writeln!(file, "# Index\n")?;
        for author in authors.as_slice() {
            writeln!(
                file,
                "- [{}](#{})",
                author.0.to_lowercase().replace(' ', "-"),
                author.0.to_lowercase().replace(' ', "-")
            )?;
        }
        writeln!(file)?; // Add an extra newline

        for author in authors {
            // Write the author's name as a Markdown heading
            writeln!(file, "\n# {}\n", author)?;

            // Get the quotes for the author
            if let Some(quotes) = self.get(author) {
                // Group quotes by book title
                let mut quotes_by_book: BTreeMap<&Book, Vec<&Quote>> = BTreeMap::new();
                for quote in quotes {
                    quotes_by_book.entry(&quote.book).or_default().push(quote);
                }

                // Write each book title and its quotes
                for (book, book_quotes) in quotes_by_book {
                    writeln!(file, "## {}\n", book)?;
                    for quote in book_quotes {
                        writeln!(file, "- \"{} {}\"", quote.quote, quote.hash)?;
                    }
                    writeln!(file)?; // Add an extra newline for spacing
                }
            }
        }
        Ok(())
    }

    pub fn write_quotes_to_file(self, file: &mut impl std::io::Write) -> anyhow::Result<()> {
        let authors = self.authors();

        // Write the index at the top of the file
        writeln!(file, "# Index\n")?;
        for author in authors.as_slice() {
            writeln!(
                file,
                "- [{}](#{})",
                author.0.to_lowercase().replace(' ', "-"),
                author.0.to_lowercase().replace(' ', "-")
            )?;
        }
        writeln!(file)?; // Add an extra newline

        for author in authors {
            // Write the author's name as a Markdown heading
            writeln!(file, "\n# {}\n", author)?;

            // Get the quotes for the author
            if let Some(quotes) = self.get(author) {
                // Group quotes by book title
                let mut quotes_by_book: BTreeMap<&Book, Vec<String>> = BTreeMap::new();
                for quote in quotes {
                    quotes_by_book
                        .entry(&quote.book)
                        .or_default()
                        .push(quote.quote.clone());
                }

                // Write each book title and its quotes
                for (book, book_quotes) in quotes_by_book {
                    writeln!(file, "## {}\n", book)?;
                    for quote in book_quotes {
                        writeln!(file, "- \"{}\"", quote)?;
                    }
                    writeln!(file)?; // Add an extra newline for spacing
                }
            }
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_author_success() {
        let input = "Indigno de ser humano (Osamu Dazai)".to_string();
        let expected_author = "Osamu Dazai".into();

        let result = Author::try_from(&input).expect("Author should be found");

        assert_eq!(result, expected_author);
    }

    #[test]
    fn test_try_author_success_special_char_in_name() {
        let input = "El idiota (Fiódor Dostoyevski)".to_string();
        let expected_author = "Fiódor Dostoyevski".into();

        let result = Author::try_from(&input).expect("Author should be found");

        assert_eq!(result, expected_author);
    }

    #[test]
    fn test_try_author_failure() {
        let input = "Indigno de ser humano".to_string(); // No author provided
        let result = Author::try_from(&input);

        assert!(
            result.is_err(),
            "Expected an error when author is not found"
        );
    }

    #[test]
    fn test_try_book_success() {
        let input = "Indigno de ser humano (Osamu Dazai)".to_string();
        let expected_book = "Indigno de ser humano".into();

        let result = Book::try_from(&input).expect("Book should be found");

        assert_eq!(result, expected_book);
    }

    #[test]
    fn test_try_book_failure() {
        let input = "(Osamu Dazai)".to_string(); // No book title provided
        let result = Book::try_from(&input);

        assert!(
            result.is_err(),
            "Expected an error when book title is not found"
        );
    }

    #[test]
    fn test_try_book_with_special_characters() {
        let input = "¡Indigno de ser humano! (Osamu Dazai)".to_string();
        let expected_book = "¡Indigno de ser humano!".into();

        let result = Book::try_from(&input).expect("Book should be found");

        assert_eq!(result, expected_book);
    }

    #[test]
    fn test_try_author_with_extra_spaces() {
        let input = "Indigno de ser humano ( Osamu Dazai )".to_string();
        let expected_author = "Osamu Dazai".into();

        let result = Author::try_from(&input).expect("Author should be found");

        assert_eq!(result, expected_author);
    }
}
