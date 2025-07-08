use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RadiusAttributeDef {
    pub name: String,
    pub code: u32,
    pub vendor: Option<u32>,
    pub data_type: String,
}

#[derive(Debug)]
pub struct Dictionary {
    pub attributes: HashMap<u32, RadiusAttributeDef>,
    pub vendors: HashMap<String, u32>,
}

impl Dictionary {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut attributes = HashMap::new();
        let mut vendors = HashMap::new();
        let mut visited = HashSet::new();

        fn parse_number(s: &str) -> Result<u32, String> {
            let s = s.trim();
            if s.starts_with("0x") || s.starts_with("0X") {
                u32::from_str_radix(&s[2..], 16)
                    .map_err(|e| format!("Invalid hex '{}': {}", s, e))
            } else {
                s.parse::<u32>()
                    .map_err(|e| format!("Invalid number '{}': {}", s, e))
            }
        }

        fn parse_file(
            path: PathBuf,
            attributes: &mut HashMap<u32, RadiusAttributeDef>,
            vendors: &mut HashMap<String, u32>,
            visited: &mut HashSet<PathBuf>,
        ) -> Result<(), String> {
            if !visited.insert(path.clone()) {
                return Ok(()); // Prevent cyclic includes
            }

            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;

            for (lineno, line) in content.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if line.starts_with("$INCLUDE") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() == 2 {
                        let include_path = path.parent().unwrap().join(parts[1]);
                        parse_file(include_path, attributes, vendors, visited)?;
                    }
                    continue;
                }

                if line.starts_with("ATTRIBUTE") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        if parts[2].contains('.') {
                            // eprintln!(
                            //     "⚠️ Skipping unsupported dotted ATTRIBUTE code '{}' in file {:?} at line {}",
                            //     parts[2], path, lineno + 1
                            // );
                            continue;
                        }
                        let name = parts[1].to_string();
                        let code = match parse_number(parts[2]) {
                            Ok(code) => code,
                            Err(e) => {
                                eprintln!(
                                    "❌ Error: {} in file {:?} at line {}",
                                    e, path, lineno + 1
                                );
                                continue;
                            }
                        };
                        let data_type = parts[3].to_string();
                        attributes.insert(
                            code,
                            RadiusAttributeDef {
                                name,
                                code,
                                vendor: None,
                                data_type,
                            },
                        );
                    }
                    continue;
                }

                if line.starts_with("VENDOR") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        if parts[2].contains('.') {
                            // eprintln!(
                            //     "⚠️ Skipping unsupported dotted VENDOR ID '{}' in file {:?} at line {}",
                            //     parts[2], path, lineno + 1
                            // );
                            continue;
                        }
                        let name = parts[1].to_string();
                        let id = match parse_number(parts[2]) {
                            Ok(id) => id,
                            Err(e) => {
                                eprintln!(
                                    "❌ Error: {} in file {:?} at line {}",
                                    e, path, lineno + 1
                                );
                                continue;
                            }
                        };
                        vendors.insert(name, id);
                    }
                    continue;
                }

                // Support other directives like BEGIN-VENDOR, VALUE, etc., as needed.
            }

            Ok(())
        }

        parse_file(path.as_ref().to_path_buf(), &mut attributes, &mut vendors, &mut visited)?;
        Ok(Dictionary { attributes, vendors })
    }
}
