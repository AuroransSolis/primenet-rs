use std::fs::{read_dir, File};
use std::path::Path;

pub fn directory_validator(s: String) -> Result<(), String> {
    let path = Path::new(&s);
    if path.exists() {
        if path.is_dir() {
            // Check to make sure the user can see the contents of the directory
            match read_dir(path) {
                Ok(iter) => {
                    // Check to make sure that the directory contains worktodo.txt/worktodo.ini
                    // (.txt for mfakto/mfaktc, .ini for Mlucas) and results.txt
                    let files = iter
                        .map(|entry_result| match entry_result {
                            Ok(entry) => entry.path().display().to_string(),
                            Err(_) => "".to_string(),
                        })
                        .collect::<Vec<_>>();
                    let mut has_worktodo_txt = false;
                    let mut has_worktodo_ini = false;
                    let mut has_results_txt = false;
                    for file in &files {
                        if (has_worktodo_txt || has_worktodo_ini) && has_results_txt {
                            break;
                        } else {
                            if file.ends_with("worktodo.txt") {
                                has_worktodo_txt = true;
                            } else if file.ends_with("worktodo.ini") {
                                has_worktodo_ini = true;
                            } else if file.ends_with("results.txt") {
                                has_results_txt = true;
                            }
                        }
                    }
                    match ((has_worktodo_txt || has_worktodo_ini), has_results_txt) {
                        (true, true) => Ok(()),
                        (false, false) => Err(format!(
                            "Directory '{}' missing worktodo.txt/worktodo.ini and results.txt",
                            s
                        )),
                        (true, false) => Err(format!("Directory '{}' missing results.txt", s)),
                        (false, true) => Err(format!(
                            "Directory '{}' missing worktodo.txt/worktodo.ini",
                            s
                        )),
                    }
                }
                Err(e) => Err(format!("Failed to open directory '{}': {}", s, e)),
            }
        } else {
            Err(format!("Path '{}' does not point to a directory.", s))
        }
    } else {
        Err(format!("Path '{}' does not exist.", s))
    }
}

pub fn numeric_validator(s: String) -> Result<(), String> {
    if s.chars().all(|c| c.is_ascii_digit()) {
        s.parse::<usize>()
            .map(|_| ())
            .map_err(|e| format!("Invalid number: '{}'. Details: {}", s, e))
    } else {
        Err(format!("Input '{}' is not all ASCII decimal digits.", s))
    }
}

pub fn p95_username_validator(s: String) -> Result<(), String> {
    if s.is_ascii() {
        if s.chars()
            .any(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        {
            Err(format!(
                "Username '{}' contains a character that is not alphanumeric, '-', or '_'.",
                s
            ))
        } else {
            Ok(())
        }
    } else {
        Err(format!("Username '{}' is not ASCII.", s))
    }
}

pub fn file_validator(s: String) -> Result<(), String> {
    let path = Path::new(&s);
    if path.exists() {
        if path.is_file() {
            match File::open(&path) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Error opening file '{}': {}", s, e)),
            }
        } else {
            Err(format!("Path '{}' does not point to a file.", s))
        }
    } else {
        Err(format!("Path '{}' does not exist.", s))
    }
}

pub fn gpu72_username_validator(s: String) -> Result<(), String> {
    if s.is_ascii() {
        Ok(())
    } else {
        Err(format!("Username '{}' is not ASCII.", s))
    }
}

pub fn f32_validator(s: String) -> Result<(), String> {
    s.parse::<f32>()
        .map(|_| ())
        .map_err(|e| format!("Invalid f32: '{}'. Details: {}", s, e))
}
