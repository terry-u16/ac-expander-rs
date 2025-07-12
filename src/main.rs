mod expander;

use anyhow::{Result, ensure};
use arboard::Clipboard;
use clap::Parser;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file to expand and format
    #[arg(value_name = "FILE")]
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.input.exists() {
        eprintln!("File '{}' does not exist", args.input.display());
        std::process::exit(1);
    }

    format_all(&args.input)?;
    let content = expand_content(&args.input)?;
    let content = format_content(content)?;
    copy_to_clipboard(content);

    Ok(())
}

fn format_all(input: &PathBuf) -> Result<()> {
    let status = std::process::Command::new("rustfmt")
        .arg(input)
        .status()
        .expect("Failed to format files");

    ensure!(status.success(), "Failed to format files");

    Ok(())
}

fn expand_content(input: &PathBuf) -> Result<String> {
    let mut content = String::new();
    let base_dir = input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    expander::expand(input, &base_dir, &mut content)?;

    Ok(content)
}

fn format_content(content: String) -> Result<String> {
    let mut tempfile = NamedTempFile::new()?;
    tempfile.write_all(content.as_bytes())?;

    let status = std::process::Command::new("rustfmt")
        .arg(tempfile.path())
        .status()
        .expect("Failed to format the output file");

    ensure!(status.success(), "Failed to format the output file");

    let mut tempfile = tempfile.reopen()?;
    let mut content = String::new();
    tempfile.read_to_string(&mut content)?;

    Ok(content)
}

fn copy_to_clipboard(content: String) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(content).unwrap();
    println!("Content copied to clipboard.");
}
