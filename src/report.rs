use crate::config::EffectiveConfig;
use crate::git;
use crate::model::{
    AuthorStat, CommitRecord, Dashboard, PathStat, RepoSummary, Report, ReportMetadata,
    TimeseriesPoint,
};
use crate::parser;
use std::collections::BTreeMap;

pub fn build_report(config: &EffectiveConfig) -> Result<Report, String> {
    let raw = git::git_log_numstat(
        &config.repo,
        config.branch.as_deref(),
        config.since.as_deref(),
        config.until.as_deref(),
    )?;
    let commits = parser::parse_git_log(&raw);
    let branch = git::current_branch(&config.repo)?;
    let authors = aggregate_authors(&commits, config);
    let summary = summarize(&authors);

    Ok(Report {
        meta: ReportMetadata {
            repo: config.repo.display().to_string(),
            branch,
            since: config.since.clone(),
            until: config.until.clone(),
        },
        summary,
        authors,
    })
}

pub fn build_dashboard(config: &EffectiveConfig) -> Result<Dashboard, String> {
    let raw = git::git_log_numstat(
        &config.repo,
        config.branch.as_deref(),
        config.since.as_deref(),
        config.until.as_deref(),
    )?;
    let commits = parser::parse_git_log(&raw);
    let branch = git::current_branch(&config.repo)?;
    let authors = aggregate_authors(&commits, config);
    let summary = summarize(&authors);
    let timeseries = aggregate_timeseries(&commits, config);
    let paths = aggregate_paths(&commits, config);

    Ok(Dashboard {
        report: Report {
            meta: ReportMetadata {
                repo: config.repo.display().to_string(),
                branch,
                since: config.since.clone(),
                until: config.until.clone(),
            },
            summary,
            authors,
        },
        timeseries,
        paths,
    })
}

fn aggregate_authors(commits: &[CommitRecord], config: &EffectiveConfig) -> Vec<AuthorStat> {
    let mut authors: BTreeMap<String, AuthorStat> = BTreeMap::new();

    for commit in commits {
        if config.no_merge && commit.is_merge {
            continue;
        }
        let entry = authors
            .entry(commit.author_email.clone())
            .or_insert_with(|| AuthorStat {
                name: commit.author_name.clone(),
                email: commit.author_email.clone(),
                ..AuthorStat::default()
            });

        entry.commit_count += 1;
        if commit.is_merge {
            entry.merge_commit_count += 1;
        } else {
            entry.effective_commit_count += 1;
        }

        for file in &commit.files {
            if file.is_binary || config.should_filter(&file.path) {
                continue;
            }
            entry.additions += file.additions;
            entry.deletions += file.deletions;
        }
    }

    let mut authors: Vec<AuthorStat> = authors
        .into_values()
        .map(|mut author| {
            author.net_lines = author.additions as i64 - author.deletions as i64;
            author
        })
        .collect();

    authors.sort_by(|a, b| {
        b.effective_commit_count
            .cmp(&a.effective_commit_count)
            .then_with(|| b.commit_count.cmp(&a.commit_count))
            .then_with(|| a.email.cmp(&b.email))
    });
    authors
}

fn summarize(authors: &[AuthorStat]) -> RepoSummary {
    let mut summary = RepoSummary {
        total_authors: authors.len() as u64,
        ..RepoSummary::default()
    };
    for author in authors {
        summary.total_commits += author.commit_count;
        summary.merge_commits += author.merge_commit_count;
        summary.effective_commits += author.effective_commit_count;
        summary.total_additions += author.additions;
        summary.total_deletions += author.deletions;
    }
    summary.net_lines = summary.total_additions as i64 - summary.total_deletions as i64;
    summary
}

fn aggregate_timeseries(
    commits: &[CommitRecord],
    config: &EffectiveConfig,
) -> Vec<TimeseriesPoint> {
    let mut by_day: BTreeMap<String, TimeseriesPoint> = BTreeMap::new();

    for commit in commits {
        if config.no_merge && commit.is_merge {
            continue;
        }

        let day = commit.date.clone();
        let entry = by_day
            .entry(day.clone())
            .or_insert_with(|| TimeseriesPoint {
                date: day,
                ..TimeseriesPoint::default()
            });
        entry.commits += 1;

        for file in &commit.files {
            if file.is_binary || config.should_filter(&file.path) {
                continue;
            }
            entry.additions += file.additions;
            entry.deletions += file.deletions;
        }
        entry.net_lines = entry.additions as i64 - entry.deletions as i64;
    }

    by_day.into_values().collect()
}

fn aggregate_paths(commits: &[CommitRecord], config: &EffectiveConfig) -> Vec<PathStat> {
    let mut path_map: BTreeMap<String, PathStat> = BTreeMap::new();

    for commit in commits {
        if config.no_merge && commit.is_merge {
            continue;
        }

        for file in &commit.files {
            if file.is_binary || config.should_filter(&file.path) {
                continue;
            }

            let entry = path_map
                .entry(file.path.clone())
                .or_insert_with(|| PathStat {
                    path: file.path.clone(),
                    ..PathStat::default()
                });
            entry.commits += 1;
            entry.additions += file.additions;
            entry.deletions += file.deletions;
            entry.net_lines = entry.additions as i64 - entry.deletions as i64;
        }
    }

    let mut paths: Vec<PathStat> = path_map.into_values().collect();
    paths.sort_by(|a, b| {
        b.additions
            .cmp(&a.additions)
            .then_with(|| b.commits.cmp(&a.commits))
            .then_with(|| a.path.cmp(&b.path))
    });
    paths.truncate(20);
    paths
}
