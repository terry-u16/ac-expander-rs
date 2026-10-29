use anyhow::{Context, Result};
use regex::Regex;
use std::{ops::Range, path::PathBuf};
use syn::{spanned::Spanned, visit::Visit};

pub(super) fn expand(path: &PathBuf, base_dir: &PathBuf, content: &mut String) -> Result<()> {
    let regex = Regex::new(r"mod\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;\s*$").unwrap();
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file '{}'", path.display()))?;
    let source = remove_test_modules(&source)
        .with_context(|| format!("Failed to parse file '{}'", path.display()))?;

    for line in source.lines() {
        if let Some(caps) = regex.captures(&line) {
            let module_name = &caps[1];
            let module_path = base_dir.join(format!("{module_name}.rs"));
            let base_dir = base_dir.join(module_name);

            content.push_str(&line.replace(";", ""));
            content.push_str(" {\n");
            expand(&module_path, &base_dir, content)?;
            content.push_str("}\n");
        } else {
            content.push_str(line);
            content.push('\n');
        }
    }

    Ok(())
}

// Remove only module items with an explicit #[cfg(test)]. Keep the original
// source outside those items, including comments and macro token streams.
fn remove_test_modules(source: &str) -> Result<String> {
    let file = syn::parse_file(source)?;
    let mut collector = TestModules {
        source,
        line_starts: std::iter::once(0)
            .chain(source.match_indices('\n').map(|(offset, _)| offset + 1))
            .collect(),
        ranges: Vec::new(),
    };
    collector.visit_file(&file);

    let mut result = source.to_owned();
    // Visit order follows source order. Reverse it to preserve earlier offsets.
    for range in collector.ranges.into_iter().rev() {
        result.replace_range(range, "");
    }
    Ok(result)
}

struct TestModules<'a> {
    source: &'a str,
    line_starts: Vec<usize>,
    ranges: Vec<Range<usize>>,
}

impl TestModules<'_> {
    fn offset(&self, location: proc_macro2::LineColumn) -> usize {
        let start = self.line_starts[location.line - 1];
        // proc_macro2 columns count characters, while String ranges use bytes.
        start
            + self.source[start..]
                .char_indices()
                .nth(location.column)
                .map_or(self.source.len() - start, |(offset, _)| offset)
    }
}

impl<'ast> Visit<'ast> for TestModules<'_> {
    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        let is_test = module.attrs.iter().any(|attr| {
            attr.path().is_ident("cfg")
                && attr
                    .parse_args::<syn::Path>()
                    .is_ok_and(|path| path.is_ident("test"))
        });
        if is_test {
            let span = module.span();
            self.ranges
                .push(self.offset(span.start())..self.offset(span.end()));
        } else {
            syn::visit::visit_item_mod(self, module);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_inline_modules_with_attributes_and_nested_braces() {
        let source = r###"
// preserved comment
#[allow(dead_code)]
#[cfg(test)]
mod checks {
    const TEXT: &str = r#"} {"#;
    /* } */
    #[test]
    fn check() { assert!(true); }
    #[cfg(test)]
    mod nested {}
}
fn main() {}
"###;
        assert_eq!(
            remove_test_modules(source).unwrap(),
            "\n// preserved comment\n\nfn main() {}\n"
        );
    }

    #[test]
    fn preserves_other_conditions_names_and_macro_contents() {
        let source = r##"
mod tests {}
#[cfg(not(test))]
mod production {}
#[cfg(any(test, feature = "extra"))]
mod conditional {}
macro_rules! sample { () => { #[cfg(test)] mod generated {} }; }
const TEXT: &str = "#[cfg(test)] mod text {}";
"##;
        assert_eq!(remove_test_modules(source).unwrap(), source);
    }

    #[test]
    fn removes_nested_modules_with_unicode_and_crlf() {
        let source = "mod outer { const TEXT: &str = \"日本語\"; #[cfg(\r\n test\r\n)] pub mod checks {} fn keep() {} }\r\n";
        assert_eq!(
            remove_test_modules(source).unwrap(),
            "mod outer { const TEXT: &str = \"日本語\";  fn keep() {} }\r\n"
        );
    }

    #[test]
    fn expands_normal_modules_without_reading_test_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::write(
            root.join("main.rs"),
            "#[cfg(test)]\nmod missing;\nmod math;\nfn main() {}\n",
        )
        .unwrap();
        std::fs::write(
            root.join("math.rs"),
            "pub fn value() -> i32 { 42 }\n#[cfg(test)]\nmod also_missing;\n#[cfg(test)]\nmod checks { #[test] fn check() {} }\n",
        )
        .unwrap();
        let mut output = String::new();
        expand(&root.join("main.rs"), &root, &mut output).unwrap();
        assert!(output.contains("mod math {"));
        assert!(output.contains("pub fn value() -> i32 { 42 }"));
        assert!(output.contains("fn main() {}"));
        assert!(!output.contains("missing"));
        assert!(!output.contains("checks"));
        assert!(!output.contains("cfg(test)"));
        syn::parse_file(&output).unwrap();
    }
}
