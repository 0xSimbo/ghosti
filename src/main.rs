use anyhow::{Context, Result};
use clap::{Arg, Command};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use std::{
    fs,
    path::Path,
};
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ImportAlias {
    import_prefix: String,
    replace_with: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GhostiConfig {
    #[serde(default = "default_ghosti_folder")]
    ghosti_folder: String,
    #[serde(default = "default_output_folder")]
    output_folder: String,
    #[serde(default)]
    import_aliases: Vec<ImportAlias>,
    #[serde(default)]
    files_to_ignore: Vec<String>,
}

fn default_ghosti_folder() -> String {
    "_ghosti".to_string()
}

fn default_output_folder() -> String {
    "src".to_string()
}

/// Represents a transformation rule to be applied to Solidity files
#[derive(Clone)]
struct TransformRule {
    pattern: Regex,
    replacement: String,
}

impl TransformRule {
    fn new<S: AsRef<str>>(pattern: &str, replacement: S) -> Result<Self> {
        Ok(Self {
            pattern: Regex::new(pattern)?,
            replacement: replacement.as_ref().to_string(),
        })
    }

    fn apply(&self, content: &str) -> String {
        self.pattern.replace_all(content, self.replacement.as_str()).to_string()
    }
}

/// Collection of all transformation rules
fn get_transform_rules(config: &GhostiConfig) -> Result<Vec<TransformRule>> {
    let mut rules = vec![
        // Remove any line containing GhostiBase.sol
        TransformRule::new(
            r#"(?m)^.*GhostiBase\.sol.*$\n?"#,
            "",
        )?,
        
        // Remove GhostiStorage variable declarations and assignments
        TransformRule::new(
            r#"(?m)^\s*GhostiStorage\s+storage\s+\w+\s*=.*$\n?"#,
            "",
        )?,

        // Remove lines with GhostiStorage references
        TransformRule::new(
            r#"(?m)^.*gs\.[^;]+;.*$\n?"#,
            "",
        )?,

        // Remove lines with getGhostiStorage
        TransformRule::new(
            r#"(?m)^.*getGhostiStorage\(\).*$\n?"#,
            "",
        )?,

        // Handle contract inheritance patterns
        TransformRule::new(
            r#"(?m)(contract\s+\w+)\s+is\s+GhostiBase\s*,\s*(\w+\s*\{)"#,
            "$1 is $2"
        )?,
        
        TransformRule::new(
            r#"(?m)(contract\s+\w+\s+is\s+\w+)\s*,\s*GhostiBase\s*(\{)"#,
            "$1 $2"
        )?,
        
        TransformRule::new(
            r#"(?m)(contract\s+\w+)\s+is\s+GhostiBase\s*\{"#,
            "$1 {"
        )?,
        
        // Clean up multiple empty lines
        TransformRule::new(r#"\n{3,}"#, "\n\n")?,
    ];

    // Add import alias replacements
    for alias in config.import_aliases.clone() {
        let pattern = format!(
            r#"(?m)(import\s+[^;]*["']){}([^"']*["'])"#,
            regex::escape(&alias.import_prefix)
        );
        rules.push(TransformRule::new(&pattern, &format!("$1{}$2", alias.replace_with))?);
    }

    Ok(rules)
}

fn transform_content(content: &str, config: &GhostiConfig) -> Result<String> {
    let rules = get_transform_rules(config)?;
    let mut transformed = content.to_string();
    
    for rule in rules {
        transformed = rule.apply(&transformed);
    }
    
    Ok(transformed)
}

const GHOSTI_BASE_TEMPLATE: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

struct GhostiStorage {
    uint256 ghosti_numbersSum;
    // You can add more storage fields here...
}

contract GhostiBase {
    bytes32 constant GHOSTI_STORAGE_SLOT = keccak256("ghosti.storage");
    function getGhostiStorage() internal pure returns (GhostiStorage storage gs) {
        // This pattern is similar to diamond storage pointing
        bytes32 slot = GHOSTI_STORAGE_SLOT;
        assembly {
            gs.slot := slot
        }
    }

    function getGhostiSum() public view returns (uint256) {
        return getGhostiStorage().ghosti_numbersSum;
    }
}
"#;

const DEFAULT_TOML_TEMPLATE: &str = r#"ghosti_folder = "_ghosti"
output_folder = "src"

# Files that won't be copied during build
files_to_ignore = [
    "Test.sol",
    "Mock.sol"
]

[[import_aliases]]
import_prefix = "@/_ghosti/"
replace_with = "@/src/"
"#;

fn init_ghosti(config_path: &Path) -> Result<()> {
    // Read existing config or create new one
    let config = if config_path.exists() {
        let config_str = fs::read_to_string(config_path)?;
        toml::from_str(&config_str)?
    } else {
        println!("Creating new ghosti.toml...");
        fs::write(config_path, DEFAULT_TOML_TEMPLATE)?;
        toml::from_str(DEFAULT_TOML_TEMPLATE)?
    };

    let ghosti_config: GhostiConfig = config;
    
    // Create ghosti folder if it doesn't exist
    let ghosti_dir = Path::new(&ghosti_config.ghosti_folder);
    if !ghosti_dir.exists() {
        println!("Creating ghosti directory at {:?}...", ghosti_dir);
        fs::create_dir_all(ghosti_dir)?;
    }

    // Create GhostiBase.sol if it doesn't exist
    let ghosti_base_path = ghosti_dir.join("GhostiBase.sol");
    if !ghosti_base_path.exists() {
        println!("Creating GhostiBase.sol...");
        fs::write(&ghosti_base_path, GHOSTI_BASE_TEMPLATE)?;
    }

    println!("Ghosti initialized successfully!");
    Ok(())
}

fn print_help_message() {
    println!(r#"
ghosti - A CLI tool for managing Solidity contracts with shared storage

USAGE:
    ghosti [COMMAND]

COMMANDS:
    init    Initialize a new ghosti project
           - Creates ghosti.toml if it doesn't exist
           - Creates _ghosti folder and GhostiBase.sol
           - Example: ghosti init
           - Options: --config <path> (default: ghosti.toml)

    build   Transform ghosti contracts to their final form
           - Removes GhostiBase inheritance and storage
           - Copies transformed contracts to output directory
           - Example: ghosti build
           - Options: --config <path> (default: ghosti.toml)


CONFIG (ghosti.toml):
    ghosti_folder     Source directory containing ghosti contracts (default: _ghosti)
    output_folder     Destination directory for transformed contracts
    files_to_ignore   List of files to skip during build
    import_aliases    Import path replacements

For more information, visit: https://github.com/glowlabs-org/ghosti
"#);
}

fn main() -> Result<()> {
    let matches = Command::new("ghosti")
        .version("0.1.0")
        .author("Simon Boccara Dev <simon@glowlabs.org>")
        .about("CLI to remove GhostiBase references and copy .sol files to output directory")
        // Add a default action when no subcommand is provided
        .arg_required_else_help(true)  // This will show help if no args provided
        .subcommand(
            Command::new("init")
                .about("Initialize a new ghosti project or scaffold missing components")
                .arg(
                    Arg::new("config")
                        .short('c')
                        .long("config")
                        .value_name("FILE")
                        .help("Sets a custom config file (defaults to ghosti.toml)")
                        .required(false)
                        .default_value("ghosti.toml"),
                ),
        )
        .subcommand(
            Command::new("build")
                .about("Build the project, transforming ghosti contracts to their final form")
                .arg(
                    Arg::new("config")
                        .short('c')
                        .long("config")
                        .value_name("FILE")
                        .help("Sets a custom config file (defaults to ghosti.toml)")
                        .required(false)
                        .default_value("ghosti.toml"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            let config_path = sub_matches.get_one::<String>("config").unwrap();
            init_ghosti(Path::new(config_path))?;
        }
        Some(("build", sub_matches)) => {
            let config_path = sub_matches.get_one::<String>("config").unwrap();
            build_ghosti(config_path)?;
        }
        Some(("help", _)) | None => {
            print_help_message();
        }
        _ => {
            println!("Unknown command. Use 'ghosti help' for usage information.");
        }
    }

    Ok(())
}

// Rename the existing main logic to build_ghosti
fn build_ghosti(config_path: &str) -> Result<()> {
    let config_str = fs::read_to_string(config_path)
        .with_context(|| format!("Could not read config file at '{}'", config_path))?;
    
    let config: GhostiConfig = toml::from_str(&config_str)
        .with_context(|| format!("Failed to parse TOML in '{}'", config_path))?;
    
    // Input / output folders
    let input_dir = Path::new(&config.ghosti_folder);
    let output_dir = Path::new(&config.output_folder);

    // Create output directory if needed
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)
            .with_context(|| format!("Failed to create output directory at {:?}", output_dir))?;
    }

    // Process all .sol files
    for entry in WalkDir::new(input_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() || entry.path().extension().and_then(|s| s.to_str()) != Some("sol") {
            continue;
        }

        // Get absolute path and convert to canonical form
        let full_path = entry.path().canonicalize()?;
        let full_path_str = full_path.to_string_lossy().to_string();

        // Skip GhostiBase.sol and ignored files using full paths
        if full_path_str.ends_with("/GhostiBase.sol") || 
           config.files_to_ignore.iter().any(|ignore_path| {
               let ignore_full_path = Path::new(ignore_path).canonicalize().unwrap_or_else(|_| Path::new(ignore_path).to_path_buf());
               let ignore_path_str = ignore_full_path.to_string_lossy().to_string();
               full_path_str.ends_with(&ignore_path_str)
           }) {
            continue;
        }

        let relative_path = entry.path().strip_prefix(input_dir).unwrap();
        let destination_path = output_dir.join(relative_path);

        // Ensure parent directories exist
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Read, transform, and write the file
        let content = fs::read_to_string(entry.path())?;
        let transformed = transform_content(&content, &config)?;
        fs::write(&destination_path, transformed)?;
    }

    println!(
        "Finished ghosti transformation from {:?} to {:?}",
        input_dir, output_dir
    );

    Ok(())
}
