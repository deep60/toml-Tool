use std::{fs::File, io::Read};
use thiserror::Error;
use toml::Value;

type ToResult<T> = std::result::Result<T, ToError>;

#[derive(Error, Debug)]
pub enum ToError {
    #[error("Failed to open file \"{file_name}\": {cause}")]
    FileOpenError { file_name: String, cause: String },

    #[error("Failed to parse TOML file \"{file_name}\": {cause}")]
    TomlParseError { file_name: String, cause: String },

    #[error("Could not find pattern {pattern}")]
    PatternNotFoundError { pattern: String },
}

pub fn extract_pattern<'a>(toml_file: &'a Value, pattern: &str) -> ToResult<&'a Value> {
    if pattern.is_empty() || pattern == "." {
        return Ok(toml_file);
    }

    let pattern = pattern.trim_start_matches('.');

    pattern
        .split('.')
        .fold(Some(toml_file), |acc, key| match acc {
            Some(a) => a.get(key),
            None => None,
        })
        .ok_or_else(|| ToError::PatternNotFoundError {
            pattern: pattern.to_string(),
        })
}

pub fn load_toml_from_file(file_name: &str) -> ToResult<toml::Value> {
    let mut file = File::open(file_name).map_err(|e| ToError::FileOpenError {
        file_name: file_name.to_string(),
        cause: e.to_string(),
    })?;
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);
    toml::from_str::<Value>(&contents).map_err(|e| ToError::TomlParseError {
        file_name: file_name.to_string(),
        cause: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_pattern() {
        let toml_file = toml::from_str(
            r#"
            [package]
            test = "test"
            "#,
        )
        .unwrap();

        let x = extract_pattern(&toml_file, "package.test").unwrap();
        assert_eq!(x, &Value::String("test".to_string()));
    }

    #[test]
    fn test_fail_extract() {
        let toml_file = toml::from_str(
            r#"
            [package]
            test = "test"
            "#,
        )
        .unwrap();

        let x = extract_pattern(&toml_file, "package.test2");

        assert!(x.is_err());
        assert_eq!(
            x.unwrap_err().to_string(),
            "Could not find pattern package.test2"
        );
    }

    #[test]
    fn test_get_prop_with_many_tables() {
        let toml_file = toml::from_str(
            r#"
            [package]
            test = "test"
            [package2]
            test2 = "test2"
            "#,
        )
        .unwrap();

        let x = extract_pattern(&toml_file, "package.test").unwrap();
        assert_eq!(x, &Value::String("test".to_string()));
    }
}
