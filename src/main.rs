use anyhow::{Context, Result};
use clap::{Arg, Command};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct GhostiConfig {
    #[serde(default = "default_ghosti_folder")]
    ghosti_folder: String,
    #[serde(default = "default_output_folder")]
    output_folder: String,
}

fn default_ghosti_folder() -> String {
    "ghosti".to_string()
}

fn default_output_folder() -> String {
    "src".to_string()
}

fn main() -> Result<()> {
    // Define CLI using clap
    let matches = Command::new("ghosti")
        .version("0.1.0")
        .author("Simon Boccara Dev <simon@glowlabs.org>")
        .about("CLI to remove GhostiCore references and copy .sol files to output directory")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file (defaults to ghosti.toml)")
                .required(false)
                .default_value("ghosti.toml"),
        )
        .get_matches();

    // Read the config file
    let config_path = matches.get_one::<String>("config").unwrap();
    let config_str = fs::read_to_string(config_path)
        .with_context(|| format!("Could not read config file at '{}'", config_path))?;
    
    let ghosti_config: GhostiConfig = toml::from_str(&config_str)
        .with_context(|| format!("Failed to parse TOML in '{}'", config_path))?;
    
    // Input / output folders
    let input_dir = Path::new(&ghosti_config.ghosti_folder);
    let output_dir = Path::new(&ghosti_config.output_folder);

    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)
            .with_context(|| format!("Failed to create output directory at {:?}", output_dir))?;
    }

    // Recursively visit each .sol file in the ghosti folder
    for entry in WalkDir::new(input_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("sol") {
            let relative_path = entry.path().strip_prefix(input_dir).unwrap();
            let destination_path = output_dir.join(relative_path);

            // Ensure parent directories exist
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Read the original .sol file
            let content = fs::read_to_string(entry.path())?;

            // Transform the file content: remove references to GhostiCore
            let transformed = remove_ghosti_lines(&content);

            // Write the transformed content to the output directory
            fs::write(&destination_path, transformed)?;
        }
    }

    println!(
        "Finished ghosti transformation from {:?} to {:?}",
        input_dir, output_dir
    );

    Ok(())
}

/// Removes lines that reference GhostiCore or ghosti-related patterns.
/// Feel free to make the regex patterns more specific for real usage.
fn remove_ghosti_lines(input: &str) -> String {
    let mut result = Vec::new();
    let mut skip_mode = false; // while true, we're skipping lines until we find a semicolon
    for line in input.lines() {
        // Check if the line references GhostiCore.sol
        if line.contains("GhostiCore.sol") {
            // Skip the entire line
            continue;
        }

        // Check if the line references GhostiCore inheritance
        if line.contains("GhostiCore") {
            // Remove simple patterns such as ` is GhostiCore` or `, GhostiCore`
            let mut modified = line.replace(" is GhostiCore", "");
            modified = modified.replace(", GhostiCore", "");
            // Keep the modified version
            result.push(modified);
            continue;
        }

        // Check if we are already skipping lines due to a partial getGhostiStorage(...) call
        if skip_mode {
            // If the semicolon is found on this line, we stop skipping
            if line.contains(';') {
                skip_mode = false;
            }
            // Either way, skip writing this line
            continue;
        }

        // Check if getGhostiStorage(...) starts on this line
        if let Some(start_index) = line.find("getGhostiStorage(") {
            // If the line also contains a semicolon after that substring,
            // we remove from "getGhostiStorage(" to the semicolon, but keep any preceding code.
            if let Some(semicolon_index) = line[start_index..].find(';') {
                // Rebuild the line up to the start of getGhostiStorage(...) 
                // plus anything after the semicolon, if that is desired.
                
                // Example: remove everything from "getGhostiStorage(" to the semicolon
                let prefix = &line[..start_index];
                let suffix = &line[start_index..][(semicolon_index + 1)..]; // text after the semicolon
                let new_line = format!("{}{}", prefix, suffix);
                // If that leaves an empty line, you could skip it. We'll keep it if it has something
                let trimmed = new_line.trim();
                if !trimmed.is_empty() {
                    result.push(new_line);
                }
            } else {
                // No semicolon found on this line, so skip from getGhostiStorage( onward
                // and keep skipping subsequent lines until we see a semicolon
                skip_mode = true;
                // So we keep the portion of the line before "getGhostiStorage("
                let partial = &line[..start_index];
                let trimmed = partial.trim();
                if !trimmed.is_empty() {
                    result.push(partial.to_string());
                }
            }
            continue;
        }

        // If line doesn't match any removal rule, keep it
        result.push(line.to_string());
    }

    // Join everything back together with line breaks
    result.join("\n")
}
