use std::fs;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn unique_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let seq = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("git-report-{name}-{nanos}-{seq}"))
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

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
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

#[test]
fn web_command_serves_health_and_dashboard_data() {
    let repo = init_repo();
    commit_file(
        &repo,
        "Alice",
        "alice@example.com",
        "feat: add app",
        "src/app.ts",
        "a\nb\n",
    );
    commit_file(
        &repo,
        "Bob",
        "bob@example.com",
        "feat: add docs",
        "docs/readme.md",
        "x\ny\nz\n",
    );

    let port = free_port();
    let mut child = Command::new(binary_path())
        .args([
            "web",
            "--repo",
            repo.to_str().unwrap(),
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
        ])
        .current_dir(&repo)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert!(line.contains("Listening on"), "{line}");

    let health = reqwest::blocking::get(format!("http://127.0.0.1:{port}/api/health"))
        .unwrap()
        .text()
        .unwrap();
    assert!(health.contains("\"status\":\"ok\""), "{health}");

    let html = reqwest::blocking::get(format!("http://127.0.0.1:{port}/"))
        .unwrap()
        .text()
        .unwrap();
    assert!(html.contains("git-report web"), "{html}");

    let dashboard = reqwest::blocking::get(format!("http://127.0.0.1:{port}/api/dashboard"))
        .unwrap()
        .text()
        .unwrap();
    assert!(dashboard.contains("\"timeseries\""), "{dashboard}");
    assert!(dashboard.contains("\"paths\""), "{dashboard}");
    assert!(dashboard.contains("\"authors\""), "{dashboard}");

    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
fn web_query_filters_accept_comma_separated_and_repeated_values() {
    let repo = init_repo();
    commit_file(
        &repo,
        "Alice",
        "alice@example.com",
        "feat: add app",
        "src/app.ts",
        "a\nb\n",
    );
    commit_file(
        &repo,
        "Alice",
        "alice@example.com",
        "test: add spec",
        "src/app.test.ts",
        "spec\n",
    );
    commit_file(
        &repo,
        "Bob",
        "bob@example.com",
        "docs: add guide",
        "docs/guide.md",
        "guide\n",
    );

    let port = free_port();
    let mut child = Command::new(binary_path())
        .args([
            "web",
            "--repo",
            repo.to_str().unwrap(),
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
        ])
        .current_dir(&repo)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert!(line.contains("Listening on"), "{line}");

    let dashboard = reqwest::blocking::get(format!(
        "http://127.0.0.1:{port}/api/dashboard?exclude_dir=docs,src/generated&exclude_ext=.test.ts,.test.tsx"
    ))
    .unwrap();
    assert!(dashboard.status().is_success(), "{dashboard:?}");
    let dashboard = dashboard.text().unwrap();
    assert!(dashboard.contains("\"src/app.ts\""), "{dashboard}");
    assert!(!dashboard.contains("\"docs/guide.md\""), "{dashboard}");
    assert!(!dashboard.contains("\"src/app.test.ts\""), "{dashboard}");

    let report = reqwest::blocking::get(format!(
        "http://127.0.0.1:{port}/api/report?exclude_dir=docs&exclude_ext=.test.ts&exclude_ext=.test.tsx"
    ))
    .unwrap();
    assert!(report.status().is_success(), "{report:?}");
    let report = report.text().unwrap();
    assert!(report.contains("\"alice@example.com\""), "{report}");
    assert!(report.contains("\"bob@example.com\""), "{report}");
    assert!(report.contains("\"total_additions\":2"), "{report}");

    child.kill().unwrap();
    child.wait().unwrap();
}
