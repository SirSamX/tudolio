use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read, Write},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug)]
struct TodoEntry {
    content: String,
    done: bool,
}

#[derive(Parser)]
#[command(name = "todo", about = "Simple Todo cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add new entry (text as arguments, or piped in)
    #[command(visible_alias = "ad")]
    Add {
        /// Add the entry as already done
        #[arg(short, long)]
        done: bool,
        content: Vec<String>,
    },

    /// List all entries
    #[command(visible_alias = "ls")]
    List {
        /// Only show open entries
        #[arg(short, long, conflicts_with = "done")]
        open: bool,
        /// Only show done entries
        #[arg(short, long)]
        done: bool,
    },

    /// Mark entry as done
    #[command(visible_alias = "do")]
    Done {
        /// Number from `todo list` (or piped in)
        index: Option<usize>,
    },

    /// Mark entry as open
    #[command(visible_alias = "op")]
    Open {
        /// Number from `todo list` (or piped in)
        index: Option<usize>,
    },

    /// Remove entry
    #[command(visible_alias = "rm")]
    Remove {
        /// Number from `todo list` (or piped in)
        index: Option<usize>,
    },

    /// Clear all done entries
    Clear {
        // Clear all entries
        #[arg(short, long)]
        all: bool,
    },

    // Edit the content of an entry
    Edit {
        index: usize,
        content: Vec<String>,
    },
}

fn data_path() -> PathBuf {
    let dirs = directories::ProjectDirs::from("", "", "todo")
        .expect("Konnte Datenverzeichnis nicht bestimmen");
    fs::create_dir_all(dirs.data_dir()).ok();
    dirs.data_dir().join("todos.json")
}

fn load() -> io::Result<Vec<TodoEntry>> {
    match fs::read_to_string(data_path()) {
        Ok(s) => {
            serde_json::from_str(&s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

fn save(entries: &[TodoEntry]) -> io::Result<()> {
    let path = data_path();
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(entries)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(&tmp, json)?;
    fs::rename(tmp, path)
}

fn not_found(index: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        format!("entry {index} does not exist"),
    )
}

fn stdin_text() -> io::Result<String> {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;
    Ok(s.trim().to_string())
}

fn get_index(arg: Option<usize>) -> io::Result<usize> {
    match arg {
        Some(i) => Ok(i),
        None => stdin_text()?
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "expected a number")),
    }
}

fn print_entry(out: &mut impl io::Write, number: usize, entry: &TodoEntry) -> io::Result<()> {
    let status = if entry.done {
        "[x]".green().to_string()
    } else {
        "[ ]".yellow().to_string()
    };

    writeln!(
        out,
        "{}. {} {}",
        number.cyan().bold(),
        status,
        entry.content
    )
}

fn main() {
    if let Err(e) = run() {
        if e.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("{} {e}", "error:".red().bold());
            std::process::exit(1);
        }
    }
}

fn run() -> io::Result<()> {
    let cli = Cli::parse();
    let mut entries = load()?;

    match cli.command {
        Commands::Add { done, content } => {
            let content = if content.is_empty() {
                stdin_text()?
            } else {
                content.join(" ")
            };
            entries.push(TodoEntry { content, done });
            save(&entries)?;
            eprintln!("{}", "Added".green());
        }
        Commands::List { done, open } => {
            if entries.is_empty() {
                eprintln!("{}", "No entries".dimmed());
            }
            let mut out = io::BufWriter::new(io::stdout().lock());
            for (i, entry) in entries.iter().enumerate() {
                if (open && entry.done) || (done && !entry.done) {
                    continue;
                };
                print_entry(&mut out, i + 1, entry)?;
            }
            out.flush()?;
        }
        Commands::Done { index } => {
            let index = get_index(index)?;
            let entry = index
                .checked_sub(1)
                .and_then(|i| entries.get_mut(i))
                .ok_or_else(|| not_found(index))?;
            entry.done = true;
            save(&entries)?;
            eprintln!("{}", "Marked entry as done".green());
        }
        Commands::Open { index } => {
            let index = get_index(index)?;
            let entry = index
                .checked_sub(1)
                .and_then(|i| entries.get_mut(i))
                .ok_or_else(|| not_found(index))?;
            entry.done = false;
            save(&entries)?;
            eprintln!("{}", "Marked entry as open".green());
        }
        Commands::Remove { index } => {
            let index = get_index(index)?;
            if !(1..=entries.len()).contains(&index) {
                return Err(not_found(index));
            }
            entries.remove(index - 1);
            save(&entries)?;
            eprintln!("{}", "Removed".red());
        }
        Commands::Clear { all } => {
            let before = entries.len();
            if all {
                entries.clear();
            } else {
                entries.retain(|e| !e.done);
            }
            let removed = before - entries.len();
            if removed > 0 {
                save(&entries)?;
            }
            eprintln!("{}", format!("Cleared {removed} entries.").green());
        }
        Commands::Edit { index, content } => {
            let entry = index
                .checked_sub(1)
                .and_then(|i| entries.get_mut(i))
                .ok_or_else(|| not_found(index))?;
            entry.content = content.join(" ");
            save(&entries)?;
            eprintln!("{}", "Edited entry".green())
        }
    }

    Ok(())
}
