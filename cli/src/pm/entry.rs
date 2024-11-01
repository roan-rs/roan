use anyhow::{bail, Result};
use tracing_subscriber::fmt::format;

#[derive(Clone, Debug)]
pub struct InstallEntry {
    user: String,
    repo: String,
    branch: Option<String>,
    tag: Option<String>,
}

impl InstallEntry {
    pub fn from_string(input: String) -> Result<Self> {
        let mut user = String::new();
        let mut repo = String::new();
        let mut branch = None;
        let mut tag = None;

        let stripped = input
            .trim_start_matches("https://")
            .trim_start_matches("github.com/")
            .trim();

        let parts: Vec<&str> = stripped.split(|c| c == '#' || c == '@').collect();

        match parts.len() {
            1 => {
                let mut segments = parts[0].split('/');
                user = segments.next().unwrap_or_default().to_string();
                repo = segments.next().unwrap_or_default().to_string();
            }
            2 => {
                let mut segments = parts[0].split('/');
                user = segments.next().unwrap_or_default().to_string();
                repo = segments.next().unwrap_or_default().to_string();

                if stripped.contains('#') {
                    branch = Some(parts[1].to_string());
                } else {
                    tag = Some(parts[1].to_string());
                }
            }
            _ => bail!("Invalid GitHub repository format: {}", input),
        }

        if user.is_empty() || repo.is_empty() {
            bail!("Invalid GitHub repository format: {}", input);
        }

        Ok(Self {
            user,
            repo,
            branch,
            tag,
        })
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn repo(&self) -> &str {
        &self.repo
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    pub fn file_name(&self) -> String {
        let branch_or_tag = match (&self.branch, &self.tag) {
            (Some(branch), _) => format!("branch-{}", branch),
            (_, Some(tag)) => format!("tag-{}", tag),
            _ => String::new(),
        };

        format!(
            "{}-{}{}.tar.gz",
            self.user,
            self.repo,
            if branch_or_tag.is_empty() { String::new() } else { format!("-{}", branch_or_tag) }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::InstallEntry;

    #[test]
    fn test_format_with_https() {
        let input = "https://github.com/roan-rs/where";
        let entry = InstallEntry::from_string(input.to_string()).unwrap();
        assert_eq!(entry.user, "roan-rs");
        assert_eq!(entry.repo, "where");
        assert!(entry.branch.is_none());
        assert!(entry.tag.is_none());
    }

    #[test]
    fn test_format_without_https() {
        let input = "github.com/roan-rs/where";
        let entry = InstallEntry::from_string(input.to_string()).unwrap();
        assert_eq!(entry.user, "roan-rs");
        assert_eq!(entry.repo, "where");
        assert!(entry.branch.is_none());
        assert!(entry.tag.is_none());
    }

    #[test]
    fn test_format_short() {
        let input = "roan-rs/where";
        let entry = InstallEntry::from_string(input.to_string()).unwrap();
        assert_eq!(entry.user, "roan-rs");
        assert_eq!(entry.repo, "where");
        assert!(entry.branch.is_none());
        assert!(entry.tag.is_none());
    }

    #[test]
    fn test_format_with_branch() {
        let input = "https://github.com/roan-rs/where#main";
        let entry = InstallEntry::from_string(input.to_string()).unwrap();
        assert_eq!(entry.user, "roan-rs");
        assert_eq!(entry.repo, "where");
        assert_eq!(entry.branch.as_deref(), Some("main"));
        assert!(entry.tag.is_none());
    }

    #[test]
    fn test_format_with_tag() {
        let input = "github.com/roan-rs/where@v1.0";
        let entry = InstallEntry::from_string(input.to_string()).unwrap();
        assert_eq!(entry.user, "roan-rs");
        assert_eq!(entry.repo, "where");
        assert!(entry.branch.is_none());
        assert_eq!(entry.tag.as_deref(), Some("v1.0"));
    }

    #[test]
    fn test_format_with_branch_and_short() {
        let input = "roan-rs/where#develop";
        let entry = InstallEntry::from_string(input.to_string()).unwrap();
        assert_eq!(entry.user, "roan-rs");
        assert_eq!(entry.repo, "where");
        assert_eq!(entry.branch.as_deref(), Some("develop"));
        assert!(entry.tag.is_none());
    }

    #[test]
    fn test_format_with_tag_and_short() {
        let input = "roan-rs/where@beta";
        let entry = InstallEntry::from_string(input.to_string()).unwrap();
        assert_eq!(entry.user, "roan-rs");
        assert_eq!(entry.repo, "where");
        assert!(entry.branch.is_none());
        assert_eq!(entry.tag.as_deref(), Some("beta"));
    }

    #[test]
    fn test_invalid_format() {
        let input = "invalid_format";
        assert!(InstallEntry::from_string(input.to_string()).is_err());
    }

    #[test]
    fn test_invalid_format_missing_repo() {
        let input = "https://github.com/roan-rs/";
        assert!(InstallEntry::from_string(input.to_string()).is_err());
    }
}
