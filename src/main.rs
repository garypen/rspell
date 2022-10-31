use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;

use anyhow::Result;
use trying::trie::TrieString;

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() -> Result<()> {
    let mut trie = TrieString::<()>::new();
    // File hosts must exist in current path before this produces output
    if let Ok(lines) = read_lines("data/words_alpha.txt") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.flatten() {
            trie.insert(line.chars());
        }
    }
    println!(
        "trie contains {}, over {} atoms",
        trie.count(),
        trie.atoms()
    );
    let stdin = io::stdin();
    let lines = stdin.lock().lines();

    print!("word: ");
    io::stdout().flush()?;
    for line in lines {
        let input = line?;
        if trie.contains(input.chars()) {
            println!("ok");
        } else {
            // There may be alternatives...
            let alternatives = trie.get_alternatives(input.chars(), 5);
            if !alternatives.is_empty() {
                print!("try: ");
                let mut sep = "";
                for word in alternatives {
                    print!("{}{}", sep, word);
                    sep = ", ";
                }
                println!();
            } else {
                println!("not found");
            }
        }
        print!("word: ");
        io::stdout().flush()?;
    }
    Ok(())
}
