use std::{collections::HashSet, path::Path};

use regex::Regex;

pub fn resolve_includes(mut source: String, parent_dir: &Path) -> Result<String, std::io::Error> {
    let mut included = HashSet::new();

    let regex = Regex::new(r#"#include ([\w/\.]+)"#).unwrap();

    while let Some(regex_match) = regex.find(&source) {
        let include_arg = regex_match
            .as_str()
            .split_ascii_whitespace()
            .nth(1)
            .unwrap();

        let relative_path = Path::new(include_arg);
        let include_path = parent_dir.join(relative_path);

        if !included.contains(&include_path) {
            let include_source = std::fs::read_to_string(&include_path)?;

            source = regex.replace(&source, &include_source).to_string();
            included.insert(include_path);
        } else {
            source = regex.replace(&source, "").to_string();
        }
    }

    Ok(source)
}
