use glob::MatchOptions;
use glob::glob_with;
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
            Ok(path) => match path.canonicalize() {
                Ok(abs) => paths.push(abs),
                Err(e) => return Err(format!("Error canonicalizing path: {}", e.to_string())),
            },
            Err(e) => return Err(format!("Error reading path: {}", e.to_string())),
        }
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct TestFixture {
        temp_dir: PathBuf,
    }

    impl TestFixture {
        fn new() -> Self {
            let temp_dir = std::env::temp_dir().join("joplin_file_finder_test");
            if temp_dir.exists() {
                fs::remove_dir_all(&temp_dir).unwrap();
            }

            fs::create_dir_all(&temp_dir).unwrap();
            Self { temp_dir }
        }
        
        fn create_file(&self, name: &PathBuf, content: &str)
        {
            fs::write(self.temp_dir.join(name), content).unwrap();
        }

        fn create_sub_directory(&self, name: &str)
        {
            fs::create_dir_all(self.temp_dir.join(name)).unwrap();
        }
    }
    
    impl Drop for TestFixture {
        fn drop(&mut self) {
            if self.temp_dir.exists() {
                fs::remove_dir_all(&self.temp_dir).unwrap()
            }
        }
    }

    #[test]
    fn test_find_files() {
        
        // arrange
        let fixture = TestFixture::new();
        fixture.create_sub_directory("1");

        let a_path = fixture.temp_dir.join("a.md");
        let b_path = fixture.temp_dir.join("b.Md");
        let c_path = fixture.temp_dir.join("1").join("c.md");
        let d_path = fixture.temp_dir.join("c");

        fixture.create_file(&a_path, "a");
        fixture.create_file(&b_path, "b");
        fixture.create_file(&c_path, "c");
        fixture.create_file(&d_path, "d");
        
        // act
        let result = find_files(fixture.temp_dir.to_str().unwrap());
        
        // assert
        assert!(result.is_ok());

        let files = result.unwrap();
        assert_eq!(files.len(), 3);

        assert!(files.iter().any(|p| p == &a_path.canonicalize().unwrap()));
        assert!(files.iter().any(|p| p == &b_path.canonicalize().unwrap()));
        assert!(files.iter().any(|p| p == &c_path.canonicalize().unwrap()));
        assert!(!files.iter().any(|p| p == &d_path.canonicalize().unwrap()));
    }
}
