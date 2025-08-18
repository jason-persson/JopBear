use glob::glob_with;
use glob::MatchOptions;
use std::path::{Path, PathBuf};

pub fn find_files(dir: &str) -> Result<Vec<PathBuf>, String> {
    let path = Path::new(dir);
    if !path.exists() {
        return Err(format!("The path {dir} does not exist"));
    }

    if !path.is_dir() {
        return Err(format!("The path {dir} is not a directory"));
    }

    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let glob_result = glob_with(&format!("{dir}/**/*.md"), options)
        .map_err(|e| format!("Error while searching for files: {}", e))?;

    let mut paths = Vec::new();
    for path in glob_result {
        match path {
            Ok(path) => paths.push(path),
            Err(e) => return Err(format!("Error reading path: {}", e.to_string())),
        }
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_files() {
        let result = find_files("/Users/jason/temp").unwrap();

        println!("{:?}", result);

        let z = 0;
    }
}
