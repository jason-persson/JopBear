pub mod joplin_file;
pub use joplin_file::JoplinFile;

pub struct Config {
    pub import_path: String,
    pub export_path: String,
}

impl Config {
    pub fn build(_: &[String]) -> Result<Config, &'static str> {
        Err("Not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {
        let result = Config::build(&[]);
        assert_eq!(result.is_err(), true);
    }
}