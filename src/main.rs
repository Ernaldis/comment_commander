extern crate termion;

use clap::Parser;

use std::io::{stdin, stdout, Write};

use std::process::Command;

use termion::clear::CurrentLine as ClearLine;
use termion::cursor::Restore as RestoreCursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

/// Run commented commands from the top of a given file
///
/// Scroll up and down using the arrow keys
///
/// Press <Enter> to execute a commands
///
/// Press `q` to quit
#[derive(Parser)]
struct Cli {
    /// The path to the file to pull commands from
    path: std::path::PathBuf,
}

fn read_lines(path: &std::path::PathBuf) -> Result<Vec<String>, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<String> = content
        .lines()
        .take_while(|line: &&str| line.starts_with('#'))
        .map(|line| line[2..].trim().to_string())
        .collect();
    Ok(lines)
}

fn main() {
    let args = Cli::parse();
    let lines = match read_lines(&args.path) {
        Ok(lines) => lines,
        Err(error) => {
            eprintln!("Failed to read file: {error}");
            return;
        }
    };

    let mut stdout = match stdout().into_raw_mode() {
        Ok(stdout) => stdout,
        Err(error) => {
            eprintln!("Failed to obtain raw mode for stdout: {error}");
            return;
        }
    };
    let mut position = 0;

    let mut write_and_flush = |text: &str| {
        if let Err(error) = write!(stdout, "{text}") {
            eprintln!("Failed to write to stdout: {error}");
            std::process::exit(1);
        }
        if let Err(error) = stdout.flush() {
            eprintln!("Failed to flush stdout: {error}");
            std::process::exit(1);
        }
    };

    write_and_flush(&format!(
        "{}{}{}{}",
        termion::cursor::Save,
        ClearLine,
        lines[position],
        RestoreCursor
    ));

    for c in stdin().keys() {
        match c {
            Ok(Key::Char('q')) => {
                write_and_flush(&format!("{ClearLine}\n\r"));
                break;
            }
            Ok(Key::Up) => {
                if position > 0 {
                    position -= 1;
                } else {
                    position = lines.len() - 1;
                }
            }
            Ok(Key::Down) => {
                if position < lines.len() - 1 {
                    position += 1;
                } else {
                    position = 0;
                }
            }
            Ok(Key::Char('\n')) => {
                let args: Vec<&str> = lines[position].split_whitespace().collect();
                let output = match Command::new(args[0]).args(&args[1..]).output() {
                    Ok(output) => output,
                    Err(error) => {
                        eprintln!("Command failed: {error}");
                        return;
                    }
                };
                let output_str = String::from_utf8_lossy(&output.stdout).replace('\n', "\n\r");
                write_and_flush(&output_str);
                break;
            }
            Err(_) => eprintln!("Failed to read input key"),
            _ => eprintln!("Unhandled input key"),
        }
        // Move the cursor down one line and write some text
        write_and_flush(&format!(
            "{}{}{}",
            ClearLine, lines[position], RestoreCursor
        ));
    }
}
