use crate::model::{CommitRecord, FileStat};

pub fn parse_git_log(raw: &str) -> Vec<CommitRecord> {
    let mut commits = Vec::new();
    let mut current: Option<CommitRecord> = None;

    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("COMMIT\x1f") {
            if let Some(commit) = current.take() {
                commits.push(commit);
            }

            let parts: Vec<&str> = rest.split('\x1f').collect();
            if parts.len() < 5 {
                continue;
            }

            let subject = parts[4].to_string();
            let is_merge = subject.starts_with("Merge branch")
                || subject.starts_with("Merge remote-tracking")
                || subject.starts_with("Merge pull request");
            current = Some(CommitRecord {
                hash: parts[0].to_string(),
                author_name: parts[1].to_string(),
                author_email: parts[2].to_string(),
                date: parts[3].to_string(),
                subject,
                is_merge,
                files: Vec::new(),
            });
            continue;
        }

        let Some(commit) = current.as_mut() else {
            continue;
        };
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }
        let is_binary = parts[0] == "-" || parts[1] == "-";
        let additions = if is_binary {
            0
        } else {
            parts[0].parse::<u64>().unwrap_or(0)
        };
        let deletions = if is_binary {
            0
        } else {
            parts[1].parse::<u64>().unwrap_or(0)
        };
        commit.files.push(FileStat {
            path: parts[2].to_string(),
            additions,
            deletions,
            is_binary,
        });
    }

    if let Some(commit) = current {
        commits.push(commit);
    }

    commits
}
