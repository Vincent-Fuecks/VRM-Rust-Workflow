use serde::{Deserialize};
use std::error::Error;
use std::fs;

pub fn parse_joson_file<T>(file_path: &str) -> Result<T, Box<dyn Error>> where T: for<'a> Deserialize<'a>, {
    let data = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read config file '{:?}': {}", file_path, e))?;

    let parsed_data: T = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse JSON from '{:?}': {}", file_path, e))?;

    Ok(parsed_data)
}


pub fn get_json_as_str(file_path: &str) -> Option<String> {
    let json_str = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            return None;
        }
    };

    return Some(json_str);
}