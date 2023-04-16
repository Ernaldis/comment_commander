extern crate termion;

use clap::Parser;

use std::io::{stdin, stdout, Write};

use std::process::Command;



use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear};

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
    let content = std::fs::read_to_string(&args.path).expect("could not read file");
    let lines: Vec<String> = content
        .lines()
        .take_while(|line: &&str| line.starts_with('#'))
        .map(|line| line[2..].trim().to_string())
        .collect();

    let mut stdout = stdout().into_raw_mode().unwrap();
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
    stdout.flush().unwrap();

    for c in stdin().keys() {
        match c.unwrap() {
            Key::Char('q') => {
                write!(stdout, "{}\n\r", clear::CurrentLine).unwrap();
                stdout.flush().unwrap();
                break;
            }
            Key::Up => {
                if position > 0 {
                    position -= 1;
                } else {
                    position = lines.len() - 1;
                }
            }
            Key::Down => {
                if position < lines.len() - 1 {
                    position += 1;
                } else {
                    position = 0;
                }
            }
            Key::Char('\n') => {
                let args: Vec<&str> = lines[position].split_whitespace().collect();
                let output = Command::new(&args[0])
                    .args(&args[1..])
                    .output()
                    .expect("Command failed");
                let output_str = String::from_utf8_lossy(&output.stdout).replace('\n', "\n\r");
                write!(stdout, "{}{}", clear::CurrentLine, output_str).unwrap();
                stdout.flush().unwrap();
                break;
            }
            _ => {}
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
        stdout.flush().unwrap();
    }
}
