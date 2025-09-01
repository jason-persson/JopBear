use crate::JoplinFile;
use glob::MatchOptions;
use glob::glob_with;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::Write;
#[cfg(target_os = "macos")]
use std::os::darwin::fs::FileTimesExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::FileTimesExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn build_joplin_files<P: AsRef<Path>>(source_dir: P) -> Result<Vec<JoplinFile>, String> {
    let paths = find_files(source_dir.as_ref().to_str().unwrap())
        .map_err(|e| format!("Error finding files: {}", e))?;

    let mut joplin_files = Vec::new();
    for path in paths {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Error reading file: {}", e.to_string()))?;

        let joplin_file = JoplinFile::build(&path.strip_prefix(&source_dir).unwrap(), &content)
            .map_err(|e| format!("Error building JoplinFile: {}", e))?;

        joplin_files.push(joplin_file);
    }

    Ok(joplin_files)
}

pub fn write_joplin_files<P: AsRef<Path>>(
    target_dir: P,
    joplin_files: &[JoplinFile],
) -> Result<(), String> {
    for joplin_file in joplin_files {
        let target_path = target_dir.as_ref().join(&joplin_file.relative_path);

        if let Some(parent) = target_path.parent() {
            create_dir_all(parent)
                .map_err(|e| format!("Error creating directory: {}", e.to_string()))?;
        }

        let mut file = File::create(&target_path)
            .map_err(|e| format!("Error creating file: {}", e.to_string()))?;

        let mut content = String::new();
        content.push_str(&joplin_file.body);
        content.push_str("\n");
        if let Some(tags) = &joplin_file.tags {
            content.push_str("\n");
            content.push_str(tags);
            content.push_str("\n");
        }

        file.write_all(content.as_bytes())
            .map_err(|e| format!("Error writing file: {}", e))?;

        let created_time = SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(joplin_file.created.timestamp() as u64);
        let modified_time = SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(joplin_file.updated.timestamp() as u64);

        let mut times = std::fs::FileTimes::new()
            .set_accessed(modified_time)
            .set_modified(modified_time);
        // On macOS and Windows, also set creation time
        // Adding Windows is a bit pointless because Bear is a macOS and iOS app only
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            times = times.set_created(created_time);
        }
        file.set_times(times)
            .map_err(|e| format!("Error setting file times: {}", e.to_string()))?;
    }

    Ok(())
}

pub fn copy_resources<P: AsRef<Path>>(source_dir: P, target_dir: P) -> Result<(), String> {
    let source_resources_dir = source_dir.as_ref().join("_resources");
    let target_resources_dir = target_dir.as_ref().join("_resources");

    if !source_resources_dir.exists() {
        return Err(format!(
            "The source path: {:?} does not exist",
            source_resources_dir
        ));
    }

    if !source_resources_dir.is_dir() {
        return Err(format!(
            "The source path: {:?} is not a directory",
            source_resources_dir
        ));
    }

    copy_dir_recursively(source_resources_dir, target_resources_dir)
        .map_err(|e| format!("Error copying resources: {}", e))?;

    Ok(())
}

pub fn copy_dir_recursively<P: AsRef<Path>>(source_dir: P, target_dir: P) -> std::io::Result<()> {
    let source_dir = source_dir.as_ref();
    let target_dir = target_dir.as_ref();

    create_dir_all(target_dir)?;
    for entry in std::fs::read_dir(source_dir)? {
        let entry = entry?;
        let source = entry.path();
        let target = target_dir.join(entry.file_name());

        if source.is_dir() {
            copy_dir_recursively(&source, &target)?;
        } else {
            std::fs::copy(&source, &target)?;
        }
    }

    Ok(())
}

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

            create_dir_all(&temp_dir).unwrap();
            Self { temp_dir }
        }

        fn create_file(&self, name: &PathBuf, content: &str) {
            fs::write(self.temp_dir.join(name), content).unwrap();
        }

        fn create_sub_directory(&self, name: &str) {
            create_dir_all(self.temp_dir.join(name)).unwrap();
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
