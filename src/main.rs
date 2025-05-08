use std::fs;
use std::path::PathBuf;
use std::process::{Command};
use std::os::unix::fs::PermissionsExt;
use term_size::dimensions;
use ansi_term::Style;
use regex::Regex;
use clap::{Parser};


/// Lists available tools from path by calling them using the --help option and parsing the output
#[derive(Parser, Debug)]
#[command(name = "image_cli")]
struct Cli {
    /// An optional wrapper command (e.g. Rustody) if describing an images capabilities. If not provided, list local tools.
    #[arg(long, short)]
    wrapper: Option<String>,

    /// Path to search for local binaries (used only if wrapper is unset or empty)
    #[arg(long, short, default_value = "/usr/local/bin")]
    path: String,

}


fn list_available_tools( tools_dir:&str ) -> Vec<String> {
    let mut list: Vec<String> = fs::read_dir(tools_dir)
        .expect("Failed to read tools directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && is_executable(&path) {
                Some(path.file_name()?.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();
    list.sort_by_key(|x| x.to_lowercase());
    list
}

fn is_executable(path: &PathBuf) -> bool {
    fs::metadata(path)
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn get_brief_help(binary: &str) -> Option<String> {
    let output = Command::new(binary)
        .arg("--help")
        .output()
        .ok()?;

    if output.status.success() {
        let help_text = String::from_utf8_lossy(&output.stdout);
        
        // Split into lines and remove leading/trailing whitespace
        let lines: Vec<&str> = help_text.lines().map(|line| line.trim()).collect();

        // Define a regex to match a version pattern (e.g., 1.0.0, 1.1.0, etc.)
        let version_regex = Regex::new(r"^\S+\s+(\d+\.\d+\.\d+)$").unwrap();

        // Skip metadata lines (lines consisting of two words where the second is a version)
        for line in &lines {
            // Try to match the version pattern (e.g., "rustody 1.1.0")
            if let Some(_captures) = version_regex.captures(line) {
                // If the second word is a valid version, skip this line as it's metadata
                if lines.len() > 2 {
                    if lines[2].trim().len() > 2 {
                        return Some(lines[2].trim().to_string())
                    }else {
                        return Some("could not parse the help string".to_string())
                    }
                }

            }else {
                return Some(line.to_string())
            }
            
        }
        Some("could not parse the help string".to_string())
    } else {
         Some("no command help available".to_string())
    }
}

fn wrap_text_with_indent(text: &str, indent: usize, max_width: usize) -> String {
    let mut result = String::new();
    let mut line = String::new();
    let words = text.split_whitespace();
    let mut width = 0;
    for word in words {
        if width + word.len() + 1 > max_width {
            result.push_str(&format!("\n{:indent$}{}", "", line, indent = indent));
            line = word.to_string();
            width = word.len();
        } else {
            if !line.is_empty() {
                line.push(' ');
                width += 1;
            }
            line.push_str(word);
            width += word.len();
        }
    }
    if !line.is_empty() {
        result.push_str(&format!("\n{:indent$}{}", "", line, indent = indent));
    }
    result.trim_start().to_string()
}

fn print_summary( wrapper:&str, path:&str ) {
    let width = dimensions().map(|(w, _)| w).unwrap_or(80);
    let indent = 20;
    println!("Available {wrapper} tools:\n");

    for tool in list_available_tools( path ) {
        let styled_tool = Style::new().bold().paint(format!("{:<indent$}", tool, indent = indent));
        let help = get_brief_help(&tool).unwrap_or("(no help available)".to_string());
        let wrapped_help = wrap_text_with_indent(&help, indent, width - indent);

        if tool.len() >= indent {
            println!("{}\n{:indent$}{}", styled_tool, "", wrapped_help, indent = indent);
        } else {
            println!("{}{:<indent$}", styled_tool, wrapped_help, indent = indent - tool.len() );
        }
    }
    println!("\nUsage: {wrapper} <tool> [args...]\nFor help on a tool: {wrapper} <tool> --help");
}


fn main() {
    let cli = Cli::parse();
    let wrapper = match cli.wrapper{
        Some(n) => n,
        None => "".to_string(),
    };
    print_summary( &wrapper, &cli.path );
}
