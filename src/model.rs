use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FileStat {
    pub path: String,
    pub additions: u64,
    pub deletions: u64,
    pub is_binary: bool,
}

#[derive(Debug, Clone)]
pub struct CommitRecord {
    pub hash: String,
    pub author_name: String,
    pub author_email: String,
    pub date: String,
    pub subject: String,
    pub is_merge: bool,
    pub files: Vec<FileStat>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AuthorStat {
    pub name: String,
    pub email: String,
    pub commit_count: u64,
    pub merge_commit_count: u64,
    pub effective_commit_count: u64,
    pub additions: u64,
    pub deletions: u64,
    pub net_lines: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RepoSummary {
    pub total_authors: u64,
    pub total_commits: u64,
    pub merge_commits: u64,
    pub effective_commits: u64,
    pub total_additions: u64,
    pub total_deletions: u64,
    pub net_lines: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportMetadata {
    pub repo: String,
    pub branch: String,
    pub since: Option<String>,
    pub until: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub meta: ReportMetadata,
    pub summary: RepoSummary,
    pub authors: Vec<AuthorStat>,
}
