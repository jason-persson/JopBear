use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct JoplinFile {
    pub title: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,

    pub front_matter: String,
    pub front_matter_start_pos: usize,
    pub front_matter_end_pos: usize,

    pub body: String,

    pub tags: Option<String>,

    pub relative_path: PathBuf,
}

impl JoplinFile {
    const MARKER: &'static str = "---\n";
    const MARKER_LEN: usize = Self::MARKER.len();

    pub fn build<P: AsRef<Path>>(
        relative_path: P,
        content: &str,
    ) -> Result<JoplinFile, &'static str> {
        let front_matter_start_pos = Self::find_front_matter_start(content)?;

        let front_matter_end_pos = Self::find_front_matter_end(front_matter_start_pos, content)?;

        let front_matter = content
            .get(front_matter_start_pos..front_matter_end_pos)
            .ok_or("Could not find front matter")?;

        let body = content[front_matter_end_pos..].trim().to_string();

        let title = Self::find_title(front_matter)?;

        let created = Self::find_created(front_matter)?;
        let updated = Self::find_updated(front_matter)?;

        let relative_path = relative_path.as_ref().to_path_buf();
        let tags = Self::build_tags(&relative_path);

        Ok(JoplinFile {
            title: title.to_string(),
            created,
            updated,
            front_matter: front_matter.to_string(),
            front_matter_start_pos,
            front_matter_end_pos,
            body,
            relative_path,
            tags,
        })
    }

    fn find_front_matter_start(content: &str) -> Result<usize, &'static str> {
        content
            .find(Self::MARKER)
            .ok_or("Could not find front matter start marker")
    }

    fn find_front_matter_end(fm_start_pos: usize, content: &str) -> Result<usize, &'static str> {
        let after_start_pos = fm_start_pos + Self::MARKER_LEN;
        let content_after_start = &content
            .get(after_start_pos..)
            .ok_or("Could not find front matter after start marker")?;

        let end_relative = content_after_start
            .find(Self::MARKER)
            .ok_or("Could not find end of front matter")?;
        let end_pos = after_start_pos + end_relative + Self::MARKER_LEN;

        if end_pos > content.len() {
            Err("Could not find end of front matter")
        } else {
            Ok(end_pos)
        }
    }

    fn find_title(front_matter: &str) -> Result<&str, &'static str> {
        const TITLE_KEY: &str = "title:";
        Self::find_front_matter_value(front_matter, TITLE_KEY).ok_or("Could not find title")
    }

    fn find_created(front_matter: &str) -> Result<DateTime<Utc>, &'static str> {
        const CREATED_KEY: &str = "created:";
        let created = Self::find_front_matter_value(front_matter, CREATED_KEY)
            .ok_or("Could not find created")?;

        DateTime::parse_from_rfc3339(created)
            .map(|result| result.to_utc())
            .map_err(|_| "Could not parse created date")
    }
    fn find_updated(front_matter: &str) -> Result<DateTime<Utc>, &'static str> {
        const UPDATED_KEY: &str = "updated:";
        let updated = Self::find_front_matter_value(front_matter, UPDATED_KEY)
            .ok_or("Could not find updated")?;

        DateTime::parse_from_rfc3339(updated)
            .map(|result| result.to_utc())
            .map_err(|_| "Could not parse updated date")
    }

    fn find_front_matter_value<'a>(front_matter: &'a str, key: &'a str) -> Option<&'a str> {
        let value = front_matter.lines().find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with(key) {
                Some(trimmed[key.len()..].trim_start())
            } else {
                None
            }
        });

        match value {
            Some(value) if !value.is_empty() => Some(value),
            _ => None,
        }
    }

    fn build_tags<P: AsRef<Path>>(relative_path: P) -> Option<String> {
        let path = relative_path.as_ref();

        let tag_count = path.components().count();
        if tag_count == 0 {
            return None;
        }

        let mut tags = "#".to_string();
        path.iter().enumerate().for_each(|(i, component)| {
            let component = component.to_str().unwrap().replace(" ", "-");

            if i < tag_count - 1 {
                tags.push_str(&component);
                tags.push('/')
            } else {
                tags.push_str(component.trim_end_matches(".md"));
            }
        });

        Some(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_front_matter_start() {
        let test_cases: Vec<(&str, Result<usize, &'static str>)> = vec![
            ("---\n", Ok(0)),
            ("\n---\n", Ok(1)),
            ("", Err("Could not find front matter start marker")),
            ("---", Err("Could not find front matter start marker")),
        ];

        for (test_case, expected) in test_cases {
            let result = JoplinFile::find_front_matter_start(test_case);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_front_matter_end() {
        let test_cases: Vec<(&str, usize, Result<usize, &'static str>)> = vec![
            ("---\n blah ---\n", 0, Ok(14)),
            ("\n---\n blah\n more blah\n ---\n", 1, Ok(27)),
            ("", 0, Err("Could not find front matter after start marker")),
            (
                "---\n blah ---",
                0,
                Err("Could not find end of front matter"),
            ),
        ];

        for (test_case, start_pos, expected) in test_cases {
            let result = JoplinFile::find_front_matter_end(start_pos, test_case);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_title() {
        let test_cases: Vec<(&str, Result<&str, &'static str>)> = vec![
            ("---\ntitle: Test\n---\n", Ok("Test")),
            ("---\ntitle:   Test  \n---\n", Ok("Test")),
            ("---\ntitle:  \n---\n", Err("Could not find title")),
            ("---\n\n---", Err("Could not find title")),
        ];

        for (test_case, expected) in test_cases {
            let result = JoplinFile::find_title(test_case);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_created() {
        let test_cases: Vec<(&str, Result<DateTime<Utc>, &'static str>)> = vec![
            (
                "---\ncreated: 2024-03-07T23:22:26Z\n---\n",
                Ok(DateTime::parse_from_rfc3339("2024-03-07 23:22:26Z")
                    .unwrap()
                    .to_utc()),
            ),
            (
                "---\ncreated: 2024-03-07T23:22:26+11:00\n---\n",
                Ok(DateTime::parse_from_rfc3339("2024-03-07 23:22:26+11:00")
                    .unwrap()
                    .to_utc()),
            ),
            (
                "---\ncreated: 2024-03-07T23:22:26\n---\n",
                Err("Could not parse created date"),
            ),
            (
                "---\ncreated: 2024-03-07\n---\n",
                Err("Could not parse created date"),
            ),
            ("---\ncreated:\n---\n", Err("Could not find created")),
            ("---\n\n---\n", Err("Could not find created")),
        ];

        for (test_case, expected) in test_cases {
            let result = JoplinFile::find_created(test_case);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_updated() {
        let test_cases: Vec<(&str, Result<DateTime<Utc>, &'static str>)> = vec![
            (
                "---\nupdated: 2024-03-07T23:22:26Z\n---\n",
                Ok(DateTime::parse_from_rfc3339("2024-03-07 23:22:26Z")
                    .unwrap()
                    .to_utc()),
            ),
            (
                "---\nupdated: 2024-03-07T23:22:26+11:00\n---\n",
                Ok(DateTime::parse_from_rfc3339("2024-03-07 23:22:26+11:00")
                    .unwrap()
                    .to_utc()),
            ),
            (
                "---\nupdated: 2024-03-07T23:22:26\n---\n",
                Err("Could not parse updated date"),
            ),
            (
                "---\nupdated: 2024-03-07\n---\n",
                Err("Could not parse updated date"),
            ),
            ("---\nupdated:\n---\n", Err("Could not find updated")),
            ("---\n\n---\n", Err("Could not find updated")),
        ];

        for (test_case, expected) in test_cases {
            let result = JoplinFile::find_updated(test_case);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_build_tags() {
        let test_cases: Vec<(&str, Option<String>)> = vec![
            ("", None),
            ("blah.md", Some("#blah".to_string())),
            ("foo/bar/baz.md", Some("#foo/bar/baz".to_string())),
        ];

        for (relative_path, expected) in test_cases {
            let result = JoplinFile::build_tags(relative_path);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_build() {
        // arrange
        let test_cases: Vec<(&str, &str, &str, &str)> = vec![
            (
                "foo.md",
                "\
---
title: Test
created: 2024-03-07T23:22:26Z
updated: 2024-04-07T08:34:52Z
---\n",
                "",
                "#foo",
            ),
            (
                "blah bah/foo.md",
                "\
---
title: Test
created: 2024-03-07T23:22:26Z
updated: 2024-04-07T08:34:52Z
---

The content\n",
                "The content",
                "#blah-bah/foo",
            ),
        ];

        for (relative_path, content, body, expected_tags) in test_cases {
            // act
            let result = JoplinFile::build(relative_path, content);

            // assert
            assert!(result.is_ok());
            let joplin_file = &result.as_ref().unwrap();

            assert_eq!(
                joplin_file.front_matter,
                "---\ntitle: Test\ncreated: 2024-03-07T23:22:26Z\nupdated: 2024-04-07T08:34:52Z\n---\n".to_string()
            );
            assert_eq!(joplin_file.body, body.to_string());
            assert_eq!(joplin_file.title, "Test".to_string());
            assert_eq!(
                joplin_file.created,
                DateTime::parse_from_rfc3339("2024-03-07 23:22:26Z")
                    .unwrap()
                    .to_utc()
            );
            assert_eq!(
                joplin_file.updated,
                DateTime::parse_from_rfc3339("2024-04-07T08:34:52Z")
                    .unwrap()
                    .to_utc()
            );
            assert_eq!(joplin_file.tags, Some(expected_tags.to_string()));
        }
    }
}
