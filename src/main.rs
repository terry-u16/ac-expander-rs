mod expander;

use anyhow::{Context, Result, ensure};
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

    /// Output file to write the formatted content
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.input.exists() {
        eprintln!("File '{}' does not exist", args.input.display());
        std::process::exit(1);
    }

    let manifest_path = find_manifest_path(&args.input)?;

    format_all(&args.input, &manifest_path)?;
    let content = expand_content(&args.input)?;
    let content = format_content(content, &manifest_path)?;
    output_content(args.output.as_ref(), content)?;

    Ok(())
}

fn find_manifest_path(input: &Path) -> Result<PathBuf> {
    let dir = input.parent().unwrap_or_else(|| Path::new("."));
    let mut dir = std::fs::canonicalize(dir).context("Failed to resolve input directory")?;

    loop {
        let manifest_path = dir.join("Cargo.toml");
        if manifest_path.exists() {
            return Ok(manifest_path);
        }

        if !dir.pop() {
            anyhow::bail!("Failed to find Cargo.toml for '{}'", input.display());
        }
    }
}

fn format_all(input: &PathBuf, manifest_path: &Path) -> Result<()> {
    format_file(input, manifest_path, "Failed to format files")
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

fn format_content(content: String, manifest_path: &Path) -> Result<String> {
    let mut tempfile = NamedTempFile::new()?;
    tempfile.write_all(content.as_bytes())?;

    format_file(
        tempfile.path(),
        manifest_path,
        "Failed to format the output file",
    )?;

    let mut tempfile = tempfile.reopen()?;
    let mut content = String::new();
    tempfile.read_to_string(&mut content)?;

    Ok(content)
}

fn format_file(path: &Path, manifest_path: &Path, message: &str) -> Result<()> {
    let status = std::process::Command::new("cargo")
        .arg("fmt")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--")
        .arg(path)
        .status()
        .context("Failed to run cargo fmt")?;

    ensure!(status.success(), "{message}");

    Ok(())
}

fn output_content(output: Option<&PathBuf>, content: String) -> Result<()> {
    match output {
        Some(path) => {
            let mut file = std::fs::File::create(path).context("Failed to create output file")?;
            file.write_all(content.as_bytes())
                .context("Failed to write to output file")?;
            println!("Content written to '{}'", path.display());
        }
        None => {
            println!("{}", content);
        }
    }

    Ok(())
}
