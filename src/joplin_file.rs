pub struct JoplinFile {
    pub front_matter: String,
    pub front_matter_start_pos: usize,
    pub front_matter_end_pos: usize,
}

impl JoplinFile {
    const MARKER: &'static str = "---\n";
    const MARKER_LEN: usize = Self::MARKER.len();

    pub fn build(content: &str) -> Result<JoplinFile, &'static str> {
        let front_matter_start_pos =
            Self::find_front_matter_start(content)
                .ok_or("Could not find front matter start marker")?;

        let front_matter_end_pos = Self::find_front_matter_end(front_matter_start_pos, content)
                .ok_or("Could not find end of front matter")?;

        let front_matter =
            Self::find_front_matter(front_matter_start_pos, front_matter_end_pos, content)
                .ok_or("Could not find front matter")?;

        Ok(JoplinFile {
            front_matter: front_matter.to_string(),
            front_matter_start_pos,
            front_matter_end_pos,
        })
    }

    // TODO I think I'll inline this
    fn find_front_matter(fm_start_pos: usize, fm_end_pos: usize, content: &str) -> Option<&str> {
        content.get(fm_start_pos..fm_end_pos)
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
    fn test_build() {
        // arrange
        let test_cases = vec![
            "\
---
title: Test
---\n",
            "\
---
title: Test
---

The content\n",
        ];

        for test_case in test_cases {
            // act
            let result = JoplinFile::build(test_case);

            // assert
            assert!(result.is_ok());
            assert_eq!(result.unwrap().front_matter, "---\ntitle: Test\n---\n".to_string());
        }
    }
}
