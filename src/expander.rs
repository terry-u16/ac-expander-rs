use anyhow::{Context, Result};
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead as _, BufReader},
    path::PathBuf,
};

pub(super) fn expand(path: &PathBuf, base_dir: &PathBuf, content: &mut String) -> Result<()> {
    let regex = Regex::new(r"mod\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;\s*$").unwrap();
    let file = File::open(path).context(format!("Failed to open file '{}'", path.display()))?;

    for line in BufReader::new(file).lines() {
        let line = line?;

        if let Some(caps) = regex.captures(&line) {
            let module_name = &caps[1];
            let module_path = base_dir.join(format!("{module_name}.rs"));
            let base_dir = base_dir.join(module_name);

            content.push_str(&line[..line.len() - 1]);
            content.push_str(" {\n");
            expand(&module_path, &base_dir, content)?;
            content.push_str("}\n");
        } else {
            content.push_str(&line);
            content.push('\n');
        }
    }

    Ok(())
}
