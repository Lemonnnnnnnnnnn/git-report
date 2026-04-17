use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("git-report-{name}-{nanos}"))
}

fn run(dir: &Path, program: &str, args: &[&str]) {
    let status = Command::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap();
    assert!(status.success(), "command failed: {program} {args:?}");
}

fn run_output(dir: &Path, program: &Path, args: &[&str]) -> std::process::Output {
    Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

fn commit_file(repo: &Path, name: &str, email: &str, subject: &str, rel: &str, body: &str) {
    let path = repo.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, body).unwrap();
    run(repo, "git", &["add", "."]);
    let status = Command::new("git")
        .args([
            "commit",
            "-m",
            subject,
            "--author",
            &format!("{name} <{email}>"),
        ])
        .current_dir(repo)
        .env("GIT_AUTHOR_DATE", "2026-04-10T10:00:00+08:00")
        .env("GIT_COMMITTER_DATE", "2026-04-10T10:00:00+08:00")
        .status()
        .unwrap();
    assert!(status.success(), "git commit failed");
}

fn init_repo() -> PathBuf {
    let repo = unique_path("repo");
    fs::create_dir_all(&repo).unwrap();
    run(&repo, "git", &["init"]);
    run(&repo, "git", &["config", "user.name", "Test User"]);
    run(&repo, "git", &["config", "user.email", "test@example.com"]);
    repo
}

fn binary_path() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    let debug_dir = exe.parent().unwrap().parent().unwrap();
    debug_dir.join(if cfg!(windows) {
        "git-report.exe"
    } else {
        "git-report"
    })
}

#[test]
fn summary_json_reports_expected_totals() {
    let repo = init_repo();
    commit_file(
        &repo,
        "Alice",
        "alice@example.com",
        "feat: add app",
        "src/app.ts",
        "line1\nline2\n",
    );
    commit_file(
        &repo,
        "Bob",
        "bob@example.com",
        "test: add spec",
        "src/app.test.ts",
        "test body\n",
    );

    let output = run_output(
        &repo,
        &binary_path(),
        &[
            "summary",
            "--repo",
            repo.to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    assert!(output.status.success(), "{output:?}");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"total_authors\":2"), "{stdout}");
    assert!(stdout.contains("\"total_commits\":2"), "{stdout}");
    assert!(stdout.contains("\"total_additions\":3"), "{stdout}");
}

#[test]
fn authors_respects_configured_filters() {
    let repo = init_repo();
    commit_file(
        &repo,
        "Alice",
        "alice@example.com",
        "feat: add impl",
        "apps/operation/src/services/a.ts",
        "service\n",
    );
    commit_file(
        &repo,
        "Alice",
        "alice@example.com",
        "feat: add logic",
        "src/main.ts",
        "a\nb\nc\n",
    );

    fs::write(
        repo.join("git-report.toml"),
        r#"
[filters]
exclude_dirs = ["apps/operation/src/services/"]
"#,
    )
    .unwrap();

    let output = run_output(
        &repo,
        &binary_path(),
        &[
            "authors",
            "--repo",
            repo.to_str().unwrap(),
            "--format",
            "json",
            "--config",
            repo.join("git-report.toml").to_str().unwrap(),
        ],
    );
    assert!(output.status.success(), "{output:?}");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"effective_commit_count\":2"), "{stdout}");
    assert!(stdout.contains("\"additions\":3"), "{stdout}");
}

#[test]
fn monthly_authors_preset_writes_markdown_and_json_files() {
    let repo = init_repo();
    commit_file(
        &repo,
        "Alice",
        "alice@example.com",
        "feat: add readme",
        "README.md",
        "hello\nworld\n",
    );

    let out_dir = repo.join("out");
    let output = run_output(
        &repo,
        &binary_path(),
        &[
            "preset",
            "monthly-authors",
            "--repo",
            repo.to_str().unwrap(),
            "--output-dir",
            out_dir.to_str().unwrap(),
        ],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(out_dir.join("monthly-report.json").exists());
    assert!(out_dir.join("monthly-report.md").exists());
}

#[test]
fn monthly_authors_preset_does_not_apply_default_path_filters() {
    let repo = init_repo();
    commit_file(
        &repo,
        "Alice",
        "alice@example.com",
        "feat: add service",
        "apps/operation/src/services/a.ts",
        "service\n",
    );
    commit_file(
        &repo,
        "Alice",
        "alice@example.com",
        "test: add spec",
        "src/app.test.ts",
        "test body\n",
    );

    let out_dir = repo.join("out");
    let output = run_output(
        &repo,
        &binary_path(),
        &[
            "preset",
            "monthly-authors",
            "--repo",
            repo.to_str().unwrap(),
            "--output-dir",
            out_dir.to_str().unwrap(),
        ],
    );
    assert!(output.status.success(), "{output:?}");

    let json = fs::read_to_string(out_dir.join("monthly-report.json")).unwrap();
    assert!(json.contains("\"total_additions\": 2"), "{json}");
    assert!(json.contains("\"additions\": 2"), "{json}");
}
