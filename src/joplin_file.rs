use chrono::{DateTime, Utc};

pub struct JoplinFile {
    pub title: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,

    pub front_matter: String,
    pub front_matter_start_pos: usize,
    pub front_matter_end_pos: usize,
}

impl JoplinFile {
    const MARKER: &'static str = "---\n";
    const MARKER_LEN: usize = Self::MARKER.len();

    pub fn build(content: &str) -> Result<JoplinFile, &'static str> {
        let front_matter_start_pos = Self::find_front_matter_start(content)
            .ok_or("Could not find front matter start marker")?;

        let front_matter_end_pos = Self::find_front_matter_end(front_matter_start_pos, content)
            .ok_or("Could not find end of front matter")?;

        let front_matter = content
            .get(front_matter_start_pos..front_matter_end_pos)
            .ok_or("Could not find front matter")?;

        let title = Self::find_title(front_matter).ok_or("Could not find title")?;

        let created = Self::find_created(front_matter).ok_or("Could not find created")?;
        let updated = Self::find_updated(front_matter).ok_or("Could not find updated")?;

        Ok(JoplinFile {
            title: title.to_string(),
            created,
            updated,
            front_matter: front_matter.to_string(),
            front_matter_start_pos,
            front_matter_end_pos,
        })
    }

    fn find_front_matter_start(content: &str) -> Option<usize> {
        let start_pos = content.find(Self::MARKER)?;
        Some(start_pos)
    }

    fn find_front_matter_end(fm_start_pos: usize, content: &str) -> Option<usize> {
        let after_start_pos = fm_start_pos + Self::MARKER_LEN;
        let content_after_start = &content.get(after_start_pos..)?;

        let end_relative = content_after_start.find(Self::MARKER)?;
        let end_pos = after_start_pos + end_relative + Self::MARKER_LEN;

        if end_pos > content.len() {
            None
        } else {
            Some(end_pos)
        }
    }

    fn find_title(front_matter: &str) -> Option<&str> {
        const TITLE_KEY: &str = "title:";
        Self::find_front_matter_value(front_matter, TITLE_KEY)
    }

    fn find_created(front_matter: &str) -> Option<DateTime<Utc>> {
        const CREATED_KEY: &str = "created:";
        let created = Self::find_front_matter_value(front_matter, CREATED_KEY)?;

        DateTime::parse_from_rfc3339(created).ok().map(|result| result.to_utc())
    }
    fn find_updated(front_matter: &str) -> Option<DateTime<Utc>> {
        const UPDATED_KEY: &str = "updated:";
        let updated = Self::find_front_matter_value(front_matter, UPDATED_KEY)?;

        DateTime::parse_from_rfc3339(updated).ok().map(|result| result.to_utc())
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_front_matter_start() {
        let test_cases: Vec<(&str, Option<usize>)> = vec![
            ("---\n", Some(0)),
            ("\n---\n", Some(1)),
            ("", None),
            ("---", None),
        ];

        for (test_case, expected) in test_cases {
            let result = JoplinFile::find_front_matter_start(test_case);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_front_matter_end() {
        let test_cases: Vec<(&str, usize, Option<usize>)> = vec![
            ("---\n blah ---\n", 0, Some(14)),
            ("\n---\n blah\n more blah\n ---\n", 1, Some(27)),
            ("", 0, None),
            ("---\n blah ---", 0, None),
        ];

        for (test_case, start_pos, expected) in test_cases {
            let result = JoplinFile::find_front_matter_end(start_pos, test_case);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_title() {
        let test_cases: Vec<(&str, Option<&str>)> = vec![
            ("---\ntitle: Test\n---\n", Some("Test")),
            ("---\ntitle:   Test  \n---\n", Some("Test")),
            ("---\ntitle:  \n---\n", None),
            ("---\n\n---", None),
        ];

        for (test_case, expected) in test_cases {
            let result = JoplinFile::find_title(test_case);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_created() {
        let test_cases: Vec<(&str, Option<DateTime<Utc>>)> = vec![
            ("---\ncreated: 2024-03-07T23:22:26Z\n---\n", Some(DateTime::parse_from_rfc3339("2024-03-07 23:22:26Z").unwrap().to_utc())),
            ("---\ncreated: 2024-03-07T23:22:26+11:00\n---\n", Some(DateTime::parse_from_rfc3339("2024-03-07 23:22:26+11:00").unwrap().to_utc())),
            ("---\ncreated: 2024-03-07T23:22:26\n---\n", None),
            ("---\ncreated: 2024-03-07\n---\n", None),
            ("---\ncreated:\n---\n", None),
            ("---\n\n---\n", None),
        ];

        for (test_case, expected) in test_cases {
            let result = JoplinFile::find_created(test_case);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_updated() {
        let test_cases: Vec<(&str, Option<DateTime<Utc>>)> = vec![
            ("---\nupdated: 2024-03-07T23:22:26Z\n---\n", Some(DateTime::parse_from_rfc3339("2024-03-07 23:22:26Z").unwrap().to_utc())),
            ("---\nupdated: 2024-03-07T23:22:26+11:00\n---\n", Some(DateTime::parse_from_rfc3339("2024-03-07 23:22:26+11:00").unwrap().to_utc())),
            ("---\nupdated: 2024-03-07T23:22:26\n---\n", None),
            ("---\nupdated: 2024-03-07\n---\n", None),
            ("---\nupdated:\n---\n", None),
            ("---\n\n---\n", None),
        ];

        for (test_case, expected) in test_cases {
            let result = JoplinFile::find_updated(test_case);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_build() {
        // arrange
        let test_cases = vec![
            "\
---
title: Test
created: 2024-03-07T23:22:26Z
updated: 2024-04-07T08:34:52Z
---\n",
            "\
---
title: Test
created: 2024-03-07T23:22:26Z
updated: 2024-04-07T08:34:52Z
---

The content\n",
        ];

        for test_case in test_cases {
            // act
            let result = JoplinFile::build(test_case);

            // assert
            assert!(result.is_ok());
            let joplin_file = &result.as_ref().unwrap();

            assert_eq!(
                joplin_file.front_matter,
                "---\ntitle: Test\ncreated: 2024-03-07T23:22:26Z\nupdated: 2024-04-07T08:34:52Z\n---\n".to_string()
            );
            assert_eq!(
                joplin_file.title,
                "Test".to_string()
            );
            assert_eq!(
                joplin_file.created,
                DateTime::parse_from_rfc3339("2024-03-07 23:22:26Z").unwrap().to_utc()
            );
            assert_eq!(
                joplin_file.updated,
                DateTime::parse_from_rfc3339("2024-04-07T08:34:52Z").unwrap().to_utc()
            );
        }
    }
}
