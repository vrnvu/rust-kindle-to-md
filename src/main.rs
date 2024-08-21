use std::{
    collections::{HashMap, HashSet},
    fs::{read_to_string, File},
    io::{BufRead, BufReader},
};

use collection::{Author, Book, Collection, Quote};

pub mod collection;

fn read_lines(filename: &str) -> Vec<String> {
    let mut result = Vec::new();

    for line in read_to_string(filename).unwrap().lines() {
        result.push(line.to_string())
    }

    result
}

fn authors(collection: &Collection) -> Vec<&Author> {
    let mut authors = collection.authors();
    authors.sort();
    authors
}

fn write_quotes_to_file(
    file: &mut impl std::io::Write,
    collection: &Collection,
) -> anyhow::Result<()> {
    let authors = authors(collection);

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
        if let Some(quotes) = collection.get(author) {
            // Group quotes by book title
            let mut quotes_by_book: HashMap<&Book, Vec<&Quote>> = HashMap::new();
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

    let collection = lines
        .chunks(5)
        .flat_map(Quote::try_from)
        .filter(|quote| !filters.contains(&quote.hash))
        .collect();

    // Open the file in write mode, creating it if it doesn't exist
    let mut file_write = File::create("README.md")?;
    write_quotes_to_file(&mut file_write, &collection)?;

    Ok(())
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
