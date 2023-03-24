use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug)]
pub struct MarkupFile {
    pub markup_type: MarkupType,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum MarkupType {
    Markdown,
    Html,
}

impl FromStr for MarkupType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "md" => Ok(Self::Markdown),
            "html" => Ok(Self::Html),
            _ => Err(()),
        }
    }
}

impl MarkupType {
    #[must_use]
    pub fn file_extensions(&self) -> Vec<String> {
        match self {
            Self::Markdown => vec![
                "md".to_string(),
                "markdown".to_string(),
                "mkdown".to_string(),
                "mkdn".to_string(),
                "mkd".to_string(),
                "mdwn".to_string(),
                "mdtxt".to_string(),
                "mdtext".to_string(),
                "text".to_string(),
                "rmd".to_string(),
            ],
            Self::Html => vec!["htm".to_string(), "html".to_string(), "xhtml".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_lowercase_file_extensions() {
        for mt in [MarkupType::Markdown, MarkupType::Html] {
            for ext in mt.file_extensions() {
                assert_eq!(ext, ext.to_lowercase());
            }
        }
    }
}
