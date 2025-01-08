use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() -> std::io::Result<()> {
    let locales_dir = "locales";
    let excluded_file = "src/common/i18n.rs";
    let mut translations: HashMap<String, HashSet<String>> = HashMap::new();
    let mut all_keys: HashSet<String> = HashSet::new();

    // Collect all translation keys for each language
    for entry in WalkDir::new(locales_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "ftl")
                && e.path().to_str().map_or(true, |p| p != excluded_file)
        })
    {
        let lang = entry
            .path()
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let content = fs::read_to_string(entry.path())?;
        let keys: HashSet<String> = content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .filter_map(|line| line.split('=').next().map(|k| k.trim().to_string()))
            .collect();

        all_keys.extend(keys.clone());
        translations.insert(lang, keys);
    }

    // Check for missing translations
    let mut has_missing = false;
    for (lang, keys) in &translations {
        let missing: Vec<_> = all_keys.difference(keys).collect();
        if !missing.is_empty() {
            has_missing = true;
            println!("Missing translations in {}: ", lang);
            for key in missing {
                println!("  - {}", key);
            }
        }
    }

    if has_missing {
        std::process::exit(1);
    }

    println!("All translations are complete!");
    Ok(())
}
