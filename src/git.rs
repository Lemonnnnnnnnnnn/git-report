use std::path::Path;
use std::process::Command;

pub fn current_branch(repo: &Path) -> Result<String, String> {
    let output = run_git(repo, &["branch", "--show-current"])?;
    Ok(output.trim().to_string())
}

pub fn git_log_numstat(
    repo: &Path,
    branch: Option<&str>,
    since: Option<&str>,
    until: Option<&str>,
) -> Result<String, String> {
    let mut args = vec![
        "log".to_string(),
        "--numstat".to_string(),
        "--date=short".to_string(),
        "--pretty=format:COMMIT\x1f%H\x1f%an\x1f%ae\x1f%ad\x1f%s".to_string(),
    ];
    if let Some(since) = since {
        args.push(format!("--since={since}"));
    }
    if let Some(until) = until {
        args.push(format!("--until={until}"));
    }
    if let Some(branch) = branch {
        args.push(branch.to_string());
    }

    let ref_args: Vec<&str> = args.iter().map(String::as_str).collect();
    run_git(repo, &ref_args)
}

fn run_git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|err| format!("failed to execute git: {err}"))?;

    if output.status.success() {
        return String::from_utf8(output.stdout)
            .map_err(|err| format!("git output was not utf-8: {err}"));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("git {} failed: {}", args.join(" "), stderr.trim()))
}
