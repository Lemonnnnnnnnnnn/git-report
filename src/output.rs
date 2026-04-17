use crate::model::{AuthorStat, RepoSummary, Report};

pub fn summary_json(summary: &RepoSummary) -> Result<String, String> {
    serde_json::to_string(summary).map_err(|err| format!("failed to serialize summary: {err}"))
}

pub fn authors_json(authors: &[AuthorStat]) -> Result<String, String> {
    serde_json::to_string(authors).map_err(|err| format!("failed to serialize authors: {err}"))
}

pub fn report_json(report: &Report) -> Result<String, String> {
    serde_json::to_string_pretty(report).map_err(|err| format!("failed to serialize report: {err}"))
}

pub fn summary_table(summary: &RepoSummary) -> String {
    [
        format!("total_authors\t{}", summary.total_authors),
        format!("total_commits\t{}", summary.total_commits),
        format!("merge_commits\t{}", summary.merge_commits),
        format!("effective_commits\t{}", summary.effective_commits),
        format!("total_additions\t{}", summary.total_additions),
        format!("total_deletions\t{}", summary.total_deletions),
        format!("net_lines\t{}", summary.net_lines),
    ]
    .join("\n")
}

pub fn authors_table(authors: &[AuthorStat]) -> String {
    let mut lines = vec!["name\temail\tcommits\teffective\tadditions\tdeletions\tnet".to_string()];
    for author in authors {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            author.name,
            author.email,
            author.commit_count,
            author.effective_commit_count,
            author.additions,
            author.deletions,
            author.net_lines
        ));
    }
    lines.join("\n")
}

pub fn report_markdown(report: &Report) -> String {
    let mut lines = vec![
        "# Git Report".to_string(),
        String::new(),
        format!("**Repository**: {}", report.meta.repo),
        format!("**Branch**: {}", report.meta.branch),
    ];
    if let Some(since) = &report.meta.since {
        lines.push(format!("**Since**: {since}"));
    }
    if let Some(until) = &report.meta.until {
        lines.push(format!("**Until**: {until}"));
    }
    lines.extend([
        String::new(),
        "## Summary".to_string(),
        String::new(),
        "| Total Authors | Total Commits | Merge Commits | Effective Commits | Total Additions | Total Deletions | Net Lines |".to_string(),
        "|---|---|---|---|---|---|---|".to_string(),
        format!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            report.summary.total_authors,
            report.summary.total_commits,
            report.summary.merge_commits,
            report.summary.effective_commits,
            report.summary.total_additions,
            report.summary.total_deletions,
            report.summary.net_lines
        ),
        String::new(),
        "## By Author".to_string(),
        String::new(),
        "| Author | Email | Commits | Effective | Additions | Deletions | Net Lines |".to_string(),
        "|---|---|---|---|---|---|---|".to_string(),
    ]);
    for author in &report.authors {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            author.name,
            author.email,
            author.commit_count,
            author.effective_commit_count,
            author.additions,
            author.deletions,
            author.net_lines
        ));
    }
    lines.join("\n")
}
