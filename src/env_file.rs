use anyhow::Result;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    KeyValue {
        key: String,
        value: String,
        comment: Option<String>,
    },
    Comment(String),
    Blank,
}

#[derive(Debug)]
pub struct EnvFile {
    header: Vec<String>,
    entries: Vec<Entry>,
}

impl EnvFile {
    pub fn parse(content: &str) -> Result<Self> {
        let mut header = Vec::new();
        let mut entries = Vec::new();
        let mut found_first_key = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if !found_first_key {
                // before first key, everything goes to header
                if trimmed.is_empty() {
                    // blank lines before first key are allowed
                    continue;
                } else if trimmed.starts_with('#') {
                    // strip the # and any following space
                    let content = trimmed.strip_prefix('#').unwrap_or(trimmed);
                    let content = content.strip_prefix(' ').unwrap_or(content);
                    header.push(content.to_string());
                } else if parse_key_value(line).is_some() {
                    // found first key, so it's an entry
                    found_first_key = true;
                    entries.push(parse_line(line)?);
                } else {
                    return Err(anyhow::anyhow!(
                        "Invalid line before first key (must be comment or blank): {}",
                        line
                    ));
                }
            } else {
                // after first key, parse normally
                entries.push(parse_line(line)?);
            }
        }

        Ok(EnvFile { header, entries })
    }

    pub fn list_keys(&self) -> Vec<&str> {
        self.entries
            .iter()
            .filter_map(|entry| match entry {
                Entry::KeyValue { key, .. } => Some(key.as_str()),
                _ => None,
            })
            .collect()
    }

    pub fn get_value(&self, key: &str) -> Option<&str> {
        self.entries.iter().find_map(|entry| match entry {
            Entry::KeyValue { key: k, value, .. } if k == key => Some(value.as_str()),
            _ => None,
        })
    }

    pub fn get_comment(&self, key: &str) -> Option<&str> {
        self.entries.iter().find_map(|entry| match entry {
            Entry::KeyValue {
                key: k, comment, ..
            } if k == key => comment.as_deref(),
            _ => None,
        })
    }

    pub fn get_header(&self) -> Option<String> {
        if self.header.is_empty() {
            None
        } else {
            Some(self.header.join("\n") + "\n")
        }
    }

    pub fn set_value(&mut self, key: &str, value: &str) {
        // find existing key and update it, preserving comment
        for entry in &mut self.entries {
            if let Entry::KeyValue {
                key: k, value: v, ..
            } = entry
                && k == key
            {
                *v = value.to_string();
                return;
            }
        }

        // key not found, add new entry
        self.entries.push(Entry::KeyValue {
            key: key.to_string(),
            value: value.to_string(),
            comment: None,
        });
    }

    pub fn set_comment(&mut self, key: &str, comment: &str) {
        for entry in &mut self.entries {
            if let Entry::KeyValue {
                key: k, comment: c, ..
            } = entry
                && k == key
            {
                *c = Some(comment.to_string());
                return;
            }
        }
    }

    pub fn set_header(&mut self, header: &str) {
        self.header = header.lines().map(|s| s.to_string()).collect();
    }

    pub fn delete_key(&mut self, key: &str) {
        self.entries.retain(|entry| match entry {
            Entry::KeyValue { key: k, .. } => k != key,
            _ => true,
        });
    }

    pub fn delete_comment(&mut self, key: &str) {
        for entry in &mut self.entries {
            if let Entry::KeyValue {
                key: k, comment, ..
            } = entry
                && k == key
            {
                *comment = None;
                return;
            }
        }
    }

    pub fn delete_header(&mut self) {
        self.header.clear();
    }
}

impl fmt::Display for EnvFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // add header if present
        if !self.header.is_empty() {
            for line in &self.header {
                writeln!(f, "# {}", line)?;
            }
            writeln!(f)?;
        }

        // add entries
        for entry in &self.entries {
            match entry {
                Entry::KeyValue {
                    key,
                    value,
                    comment,
                } => {
                    write!(f, "{}={}", key, value)?;
                    if let Some(c) = comment {
                        write!(f, " # {}", c)?;
                    }
                    writeln!(f)?;
                }
                Entry::Comment(c) => {
                    writeln!(f, "{}", c)?;
                }
                Entry::Blank => {
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}

fn parse_line(line: &str) -> Result<Entry> {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return Ok(Entry::Blank);
    }

    if trimmed.starts_with('#') {
        return Ok(Entry::Comment(line.to_string()));
    }

    if let Some((key, value, comment)) = parse_key_value(line) {
        return Ok(Entry::KeyValue {
            key: key.to_string(),
            value: value.to_string(),
            comment: comment.map(|s| s.to_string()),
        });
    }

    Err(anyhow::anyhow!(
        "Invalid line (must be KEY=VALUE, comment, or blank): {}",
        line
    ))
}

fn parse_key_value(line: &str) -> Option<(&str, &str, Option<&str>)> {
    // find the first '=' sign
    let equal_pos = line.find('=')?;
    let key = line[..equal_pos].trim();

    // key must not be empty
    if key.is_empty() {
        return None;
    }

    let rest = &line[equal_pos + 1..];

    // look for comment after value
    if let Some(hash_pos) = rest.find('#') {
        let value = rest[..hash_pos].trim();
        let comment = rest[hash_pos + 1..].trim();
        Some((
            key,
            value,
            if comment.is_empty() {
                None
            } else {
                Some(comment)
            },
        ))
    } else {
        let value = rest.trim();
        Some((key, value, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let content = "KEY=value\n";
        let env = EnvFile::parse(content).unwrap();
        assert_eq!(env.list_keys(), vec!["KEY"]);
        assert_eq!(env.get_value("KEY"), Some("value"));
    }

    #[test]
    fn test_parse_with_comment() {
        let content = "KEY=value # this is a comment\n";
        let env = EnvFile::parse(content).unwrap();
        assert_eq!(env.get_value("KEY"), Some("value"));
        assert_eq!(env.get_comment("KEY"), Some("this is a comment"));
    }

    #[test]
    fn test_parse_with_header() {
        let content = "# Header line 1\n# Header line 2\n\nKEY=value\n";
        let env = EnvFile::parse(content).unwrap();
        assert_eq!(
            env.get_header(),
            Some("Header line 1\nHeader line 2\n".to_string())
        );
        assert_eq!(env.get_value("KEY"), Some("value"));
    }

    #[test]
    fn test_set_value_preserves_comment() {
        let content = "KEY=old # comment\n";
        let mut env = EnvFile::parse(content).unwrap();
        env.set_value("KEY", "new");
        assert_eq!(env.get_value("KEY"), Some("new"));
        assert_eq!(env.get_comment("KEY"), Some("comment"));
    }

    #[test]
    fn test_set_value_new_key() {
        let content = "KEY1=value1\n";
        let mut env = EnvFile::parse(content).unwrap();
        env.set_value("KEY2", "value2");
        assert_eq!(env.list_keys(), vec!["KEY1", "KEY2"]);
        assert_eq!(env.get_value("KEY2"), Some("value2"));
    }

    #[test]
    fn test_delete_key_removes_comment() {
        let content = "KEY=value # comment\n";
        let mut env = EnvFile::parse(content).unwrap();
        env.delete_key("KEY");
        assert_eq!(env.list_keys().len(), 0);
        assert_eq!(env.get_comment("KEY"), None);
    }

    #[test]
    fn test_delete_comment_only() {
        let content = "KEY=value # comment\n";
        let mut env = EnvFile::parse(content).unwrap();
        env.delete_comment("KEY");
        assert_eq!(env.get_value("KEY"), Some("value"));
        assert_eq!(env.get_comment("KEY"), None);
    }

    #[test]
    fn test_set_comment() {
        let content = "KEY=value\n";
        let mut env = EnvFile::parse(content).unwrap();
        env.set_comment("KEY", "new comment");
        assert_eq!(env.get_comment("KEY"), Some("new comment"));
    }

    #[test]
    fn test_set_header() {
        let content = "KEY=value\n";
        let mut env = EnvFile::parse(content).unwrap();
        env.set_header("New header\nLine 2");
        assert_eq!(env.get_header(), Some("New header\nLine 2\n".to_string()));
    }

    #[test]
    fn test_delete_header() {
        let content = "# Header\nKEY=value\n";
        let mut env = EnvFile::parse(content).unwrap();
        env.delete_header();
        assert_eq!(env.get_header(), None);
    }

    #[test]
    fn test_to_string_preserves_format() {
        let content =
            "# Header\n\nKEY1=value1 # comment1\nKEY2=value2\n\n# middle comment\nKEY3=value3\n";
        let env = EnvFile::parse(content).unwrap();
        let output = env.to_string();
        assert_eq!(output, content);
    }

    #[test]
    fn test_blank_lines_preserved() {
        let content = "KEY1=value1\n\nKEY2=value2\n";
        let env = EnvFile::parse(content).unwrap();
        let output = env.to_string();
        assert_eq!(output, content);
    }

    #[test]
    fn test_comment_lines_preserved() {
        let content = "KEY1=value1\n# This is a comment\nKEY2=value2\n";
        let env = EnvFile::parse(content).unwrap();
        let output = env.to_string();
        assert_eq!(output, content);
    }

    #[test]
    fn test_value_with_equals_sign() {
        let content = "KEY=value=with=equals\n";
        let env = EnvFile::parse(content).unwrap();
        assert_eq!(env.get_value("KEY"), Some("value=with=equals"));
    }

    #[test]
    fn test_empty_value() {
        let content = "KEY=\n";
        let env = EnvFile::parse(content).unwrap();
        assert_eq!(env.get_value("KEY"), Some(""));
    }

    #[test]
    fn test_value_with_spaces() {
        let content = "KEY=  value with spaces  \n";
        let env = EnvFile::parse(content).unwrap();
        assert_eq!(env.get_value("KEY"), Some("value with spaces"));
    }

    #[test]
    fn test_key_with_spaces_trimmed() {
        let content = "  KEY  =value\n";
        let env = EnvFile::parse(content).unwrap();
        assert_eq!(env.get_value("KEY"), Some("value"));
    }

    #[test]
    fn test_comment_without_space_after_hash() {
        let content = "KEY=value #comment\n";
        let env = EnvFile::parse(content).unwrap();
        assert_eq!(env.get_comment("KEY"), Some("comment"));
    }

    #[test]
    fn test_invalid_line_after_key_errors() {
        let content = "KEY=value\ninvalid line\n";
        let result = EnvFile::parse(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid line"));
    }

    #[test]
    fn test_invalid_line_before_key_errors() {
        let content = "# header\ninvalid\nKEY=value\n";
        let result = EnvFile::parse(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid line before first key")
        );
    }

    #[test]
    fn test_blank_lines_before_first_key_allowed() {
        let content = "\n\n# header\n\nKEY=value\n";
        let env = EnvFile::parse(content).unwrap();
        assert_eq!(env.get_value("KEY"), Some("value"));
    }
}
