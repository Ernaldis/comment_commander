extern crate termion;

use clap::Parser;

use std::io::{stdin, stdout, Write};

use std::process::Command;

use termion::clear;
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

fn main() {
    let args = Cli::parse();
    let content = match std::fs::read_to_string(&args.path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("Could not read file: {error}");
            return;
        }
    };

    let lines: Vec<String> = content
        .lines()
        .take_while(|line: &&str| line.starts_with('#'))
        .map(|line| line[2..].trim().to_string())
        .collect();

    let mut stdout = match stdout().into_raw_mode() {
        Ok(stdout) => stdout,
        Err(error) => {
            eprintln!("Failed to obtain raw mode for stdout: {error}");
            return;
        }
    };
    let mut position = 0;

    write!(
        stdout,
        "{}{}{}{}",
        termion::cursor::Save,
        clear::CurrentLine,
        lines[position],
        termion::cursor::Restore
    )
    .unwrap();

    if let Err(error) = stdout.flush() {
        eprintln!("Failed to flush stdout: {error}");
        return;
    }

    for c in stdin().keys() {
        match c {
            Ok(Key::Char('q')) => {
                write!(stdout, "{}\n\r", clear::CurrentLine).unwrap();
                if let Err(error) = stdout.flush() {
                    eprintln!("Failed to flush stdout: {error}");
                    return;
                }
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
                write!(stdout, "{}{}", clear::CurrentLine, output_str).unwrap();
                if let Err(error) = stdout.flush() {
                    eprintln!("Failed to flush stdout: {error}");
                    return;
                }
                break;
            }
            Err(_) => todo!(),
            _ => todo!(),
        }
        // Move the cursor down one line and write some text
        write!(
            stdout,
            "{}{}{}",
            clear::CurrentLine,
            lines[position],
            termion::cursor::Restore
        )
        .unwrap();
        if let Err(error) = stdout.flush() {
            eprintln!("Failed to flush stdout: {error}");
            return;
        }
    }
}
