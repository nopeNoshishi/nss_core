// External
use anyhow::Result;
use chrono::prelude::{DateTime, Utc};
use chrono::TimeZone;

// Internal
use super::object::Hashable;

/// **Commit Struct**
///
/// This struct represents ...
#[derive(Debug, Clone, PartialEq)]
pub struct Commit {
    pub tree_hash: String,
    pub parent: String,
    pub author: String,
    pub committer: String,
    pub date: DateTime<Utc>,
    pub message: String,
}

impl Commit {
    /// Create commit with the repo tree object, config infomation and message.
    ///
    /// This tree_hash must be in the database.
    pub fn new<S: Into<String>>(
        tree_hash: S,
        parent: S,
        author: S,
        committer: S,
        message: S,
    ) -> Result<Self> {
        Ok(Self {
            tree_hash: tree_hash.into(),
            parent: parent.into(),
            author: author.into(),
            committer: committer.into(),
            date: Utc::now(),
            message: message.into(),
        })
    }

    pub fn from_rawobject(content: &[u8]) -> Result<Self> {
        let all_line = content
            .split(|&x| x == b'\n')
            .filter(|x| x != b"")
            .map(|x| String::from_utf8(x.to_vec()).unwrap())
            .collect::<Vec<String>>();

        // TODO: Refactor！
        let mut iter = all_line[0].split_whitespace();
        iter.next();
        let tree_hash = iter.next().unwrap().to_string();

        let mut iter = all_line[1].split_whitespace();
        iter.next();
        let parent = iter.next().unwrap().to_string();

        let mut iter = all_line[2].split_whitespace();
        iter.next();
        let author = iter.next().unwrap().to_string();

        let mut iter = all_line[3].split_whitespace();
        iter.next();
        let committer = iter.next().unwrap().to_string();

        let mut iter = all_line[4].split_whitespace();
        iter.next();
        let date = iter.next().unwrap().to_string();

        let message = all_line[5].clone();

        Ok(Self {
            tree_hash,
            parent,
            author,
            committer,
            date: Utc.timestamp_opt(date.parse::<i64>()?, 0).unwrap(),
            message,
        })
    }
}

impl std::fmt::Display for Commit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let tree = format!("tree {}", self.tree_hash);
        let parent = match self.parent.as_str() {
            "None" => "parent None\n".to_string(),
            _ => format!("parent {}\n", self.parent),
        };
        let author = format!("author {}", self.author);
        let committer = format!("committer {}", self.committer);
        let date = format!("date {}", self.date.timestamp());

        write!(
            f,
            "{}\n{}{}\n{}\n{}\n\n{}\n",
            tree, parent, author, committer, date, self.message
        )
    }
}

impl Hashable for Commit {
    fn as_bytes(&self) -> Vec<u8> {
        let tree_hash = format!("tree {}", self.tree_hash);
        let parent = match self.parent.as_str() {
            "None" => "parent None\n".to_string(),
            _ => format!("parent {}\n", self.parent),
        };
        let author = format!("author {}", self.author);
        let committer = format!("committer {}", self.committer);
        let date = format!("date {}", self.date.timestamp());
        let content = format!(
            "{}\n{}{}\n{}\n{}\n\n{}\n",
            tree_hash, parent, author, committer, date, self.message
        );
        let store = format!("commit {}\0{}", content.len(), content);

        Vec::from(store.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_new() {
        let result = Commit::new(
            "c192349d0ee530038e5d925fdd701652ca755ba8",
            "a02b83cb54ba139e5c9d623a2fcf5424552946e0",
            "nopeNoshihsi",
            "nopeNoshihsi",
            "initial",
        );
        assert!(result.is_ok());

        let commit = result.unwrap();
        let time = commit.date;

        let test_commit = Commit {
            tree_hash: "c192349d0ee530038e5d925fdd701652ca755ba8".to_string(),
            parent: "a02b83cb54ba139e5c9d623a2fcf5424552946e0".to_string(),
            author: "nopeNoshihsi".to_string(),
            committer: "nopeNoshihsi".to_string(),
            date: time,
            message: "initial".to_string(),
        };

        assert_eq!(commit, test_commit);
    }

    #[test]
    fn test_commit_from_rawobject() {
        // Create a sample content as bytes
        let content = b"tree c192349d0ee530038e5d925fdd701652ca755ba8
parent a02b83cb54ba139e5c9d623a2fcf5424552946e0
author nopeNoshihsi
committer nopeNoshihsi
date 1687619045

initial
";

        // Create a Commit from the raw object content
        let commit = Commit::from_rawobject(content).unwrap();

        // Verify the Commit instance's properties
        let test_commit = Commit {
            tree_hash: "c192349d0ee530038e5d925fdd701652ca755ba8".to_string(),
            parent: "a02b83cb54ba139e5c9d623a2fcf5424552946e0".to_string(),
            author: "nopeNoshihsi".to_string(),
            committer: "nopeNoshihsi".to_string(),
            date: Utc.timestamp_opt(1687619045, 0).unwrap(),
            message: "initial".to_string(),
        };

        assert_eq!(commit, test_commit);
    }

    #[test]
    fn test_commit_as_bytes() {
        let time = Utc.timestamp_opt(1687619045, 0).unwrap();
        let commit = Commit {
            tree_hash: "c192349d0ee530038e5d925fdd701652ca755ba8".to_string(),
            parent: "a02b83cb54ba139e5c9d623a2fcf5424552946e0".to_string(),
            author: "nopeNoshihsi".to_string(),
            committer: "nopeNoshihsi".to_string(),
            date: time,
            message: "initial".to_string(),
        };

        let content = commit.as_bytes();

        let test_content = b"commit 162\0tree c192349d0ee530038e5d925fdd701652ca755ba8
parent a02b83cb54ba139e5c9d623a2fcf5424552946e0
author nopeNoshihsi
committer nopeNoshihsi
date 1687619045

initial
";
        assert_eq!(content, test_content);
    }

    #[test]
    fn test_commit_to_hash() {}

    #[test]
    fn test_commit_display() {}
}
