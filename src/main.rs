use std::env;
use std::fs::{metadata, File};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
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

fn get_history_file() -> Option<PathBuf> {
    dirs::preference_dir()
        .and_then(|mut base| {
            base.push("rspell");
            // Note: Not create_dir_all(), because we don't want to create preference
            // dirs if they don't exist.
            if metadata(base.clone()).ok().is_none() {
                std::fs::create_dir(base.clone()).ok()?
            }
            Some(base)
        })
        .map(|mut base| {
            base.push("history.txt");
            base
        })
}

fn usage() {
    eprintln!("usage: rspell <dictionary>");
    std::process::exit(1);
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        usage();
    }

    let lines = read_lines(&args[1])?;
    let mut trie = TrieString::<()>::new();

    // Consumes the iterator, returns an (Optional) String
    for line in lines.flatten() {
        trie.insert(line.chars());
    }

    println!(
        "trie contains {} entries, using {} atoms",
        trie.count(),
        trie.atoms()
    );

    let mut rl = DefaultEditor::new()?;
    if let Some(file_location) = get_history_file() {
        if let Err(e) = rl.load_history(&file_location) {
            println!("error loading history: {}", e);
        }
    }
    println!("terminate with ctrl-c or ctrl-d");
    loop {
        let readline = rl.readline("word: ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                rl.add_history_entry(line.as_str())?;
                if trie.contains(line.chars()) {
                    println!("ok");
                } else {
                    // There may be alternatives...
                    let alternatives = trie.get_alternatives(line.chars(), 5);
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
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("terminating...");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    if let Some(file_location) = get_history_file() {
        if let Err(e) = rl.save_history(&file_location) {
            println!("error saving history: {}", e);
        }
    }
    Ok(())
}
