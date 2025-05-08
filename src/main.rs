use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, exit};
use std::os::unix::fs::PermissionsExt;
use term_size::dimensions;
use ansi_term::Style;
use regex::Regex;

/// List of known binary tools shipped with the image.
/// These could be dynamically scanned from a directory as well.
const TOOLS_DIR: &str = "/usr/local/bin"; // change to match your image layout

fn list_available_tools() -> Vec<String> {
    fs::read_dir(TOOLS_DIR)
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
        .collect()
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

fn print_summary() {
    let width = dimensions().map(|(w, _)| w).unwrap_or(80);
    let indent = 20;
    println!("Available Rustody tools:\n");

    for tool in list_available_tools() {
        let styled_tool = Style::new().bold().paint(format!("{:<indent$}", tool, indent = indent));
        let help = get_brief_help(&tool).unwrap_or("(no help available)".to_string());
        let wrapped_help = wrap_text_with_indent(&help, indent, width - indent);

        if tool.len() >= indent {
            println!("{}\n{:indent$}{}", styled_tool, "", wrapped_help, indent = indent);
        } else {
            println!("{}{:<indent$}", styled_tool, wrapped_help, indent = indent - tool.len() );
        }
    }
    println!("\nUsage: Rustody <tool> [args...]\nFor help on a tool: Rustody <tool> --help");
}

fn dispatch_command(args: &[String]) {
    let binary = &args[0];
    let rest = &args[1..];

    let status = Command::new(binary)
        .args(rest)
        .status()
        .unwrap_or_else(|_| {
            eprintln!("Error: failed to run tool '{}'. Is it in $PATH?", binary);
            exit(1);
        });

    exit(status.code().unwrap_or(1));
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_summary();
    } else {
        dispatch_command(&args);
    }
}
