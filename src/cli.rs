use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{load_file_config, EffectiveConfig};
use crate::output;
use crate::report;
use crate::server::{self, ServerConfig};

#[derive(Debug, Parser)]
#[command(name = "git-report")]
#[command(about = "Generate reusable Git activity reports")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Summary(CommandOptions),
    Authors(CommandOptions),
    Report(ReportOptions),
    Web(WebOptions),
    Preset(PresetCommand),
}

#[derive(Debug, Args, Clone)]
struct SharedOptions {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long)]
    since: Option<String>,
    #[arg(long)]
    until: Option<String>,
    #[arg(long)]
    branch: Option<String>,
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    no_merge: bool,
    #[arg(long = "exclude-dir")]
    exclude_dirs: Vec<String>,
    #[arg(long = "exclude-ext")]
    exclude_extensions: Vec<String>,
}

#[derive(Debug, Args, Clone)]
struct CommandOptions {
    #[command(flatten)]
    base: SharedOptions,
    #[arg(long, default_value = "table")]
    format: OutputFormat,
}

#[derive(Debug, Args, Clone)]
struct ReportOptions {
    #[command(flatten)]
    base: CommandOptions,
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct PresetCommand {
    #[command(subcommand)]
    preset: PresetKind,
}

#[derive(Debug, Subcommand)]
enum PresetKind {
    MonthlyAuthors(PresetOptions),
}

#[derive(Debug, Args, Clone)]
struct PresetOptions {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long)]
    branch: Option<String>,
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    output_dir: Option<PathBuf>,
    #[arg(long = "exclude-dir")]
    exclude_dirs: Vec<String>,
    #[arg(long = "exclude-ext")]
    exclude_extensions: Vec<String>,
}

#[derive(Debug, Args, Clone)]
struct WebOptions {
    #[command(flatten)]
    base: SharedOptions,
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value_t = 3000)]
    port: u16,
    #[arg(long)]
    open: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Markdown,
}

pub async fn run() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Summary(options) => {
            let config = merge_command_config(&options)?;
            let report = report::build_report(&config)?;
            print_line(match options.format {
                OutputFormat::Json => output::summary_json(&report.summary)?,
                OutputFormat::Table | OutputFormat::Markdown => {
                    output::summary_table(&report.summary)
                }
            });
        }
        Commands::Authors(options) => {
            let config = merge_command_config(&options)?;
            let report = report::build_report(&config)?;
            print_line(match options.format {
                OutputFormat::Json => output::authors_json(&report.authors)?,
                OutputFormat::Table | OutputFormat::Markdown => {
                    output::authors_table(&report.authors)
                }
            });
        }
        Commands::Report(options) => {
            let config = merge_command_config(&options.base)?;
            let report = report::build_report(&config)?;
            let content = match options.base.format {
                OutputFormat::Json => output::report_json(&report)?,
                OutputFormat::Markdown | OutputFormat::Table => output::report_markdown(&report),
            };
            if let Some(path) = options.output {
                fs::write(&path, content)
                    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
            } else {
                print_line(content);
            }
        }
        Commands::Web(options) => {
            let config = merge_shared_config(&options.base)?;
            server::serve(ServerConfig {
                host: options.host,
                port: options.port,
                open: options.open,
                report: config,
            })
            .await?;
        }
        Commands::Preset(command) => match command.preset {
            PresetKind::MonthlyAuthors(options) => {
                let config = merge_preset_config(&options)?;
                let report = report::build_report(&config)?;
                let out_dir = options
                    .output_dir
                    .unwrap_or_else(|| config.repo.join("git-reports"));
                fs::create_dir_all(&out_dir)
                    .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;
                let json_path = out_dir.join("monthly-report.json");
                let md_path = out_dir.join("monthly-report.md");
                fs::write(&json_path, output::report_json(&report)?)
                    .map_err(|err| format!("failed to write {}: {err}", json_path.display()))?;
                fs::write(&md_path, output::report_markdown(&report))
                    .map_err(|err| format!("failed to write {}: {err}", md_path.display()))?;
                print_line(format!("Wrote {}", json_path.display()));
                print_line(format!("Wrote {}", md_path.display()));
            }
        },
    }
    Ok(())
}

fn merge_command_config(options: &CommandOptions) -> Result<EffectiveConfig, String> {
    let repo = canonical_repo(options.base.repo.as_deref())?;
    let file_config =
        load_file_config(resolve_config_path(options.base.config.as_deref(), &repo).as_deref())?;
    let mut exclude_dirs = file_config.filters.exclude_dirs;
    exclude_dirs.extend(options.base.exclude_dirs.clone());
    let mut exclude_extensions = file_config.filters.exclude_extensions;
    exclude_extensions.extend(options.base.exclude_extensions.clone());
    Ok(EffectiveConfig {
        repo,
        since: options.base.since.clone(),
        until: options.base.until.clone(),
        branch: options.base.branch.clone(),
        no_merge: options.base.no_merge,
        exclude_dirs,
        exclude_extensions,
    })
}

fn merge_shared_config(options: &SharedOptions) -> Result<EffectiveConfig, String> {
    let repo = canonical_repo(options.repo.as_deref())?;
    let file_config =
        load_file_config(resolve_config_path(options.config.as_deref(), &repo).as_deref())?;
    let mut exclude_dirs = file_config.filters.exclude_dirs;
    exclude_dirs.extend(options.exclude_dirs.clone());
    let mut exclude_extensions = file_config.filters.exclude_extensions;
    exclude_extensions.extend(options.exclude_extensions.clone());
    Ok(EffectiveConfig {
        repo,
        since: options.since.clone(),
        until: options.until.clone(),
        branch: options.branch.clone(),
        no_merge: options.no_merge,
        exclude_dirs,
        exclude_extensions,
    })
}

fn merge_preset_config(options: &PresetOptions) -> Result<EffectiveConfig, String> {
    let repo = canonical_repo(options.repo.as_deref())?;
    let file_config =
        load_file_config(resolve_config_path(options.config.as_deref(), &repo).as_deref())?;
    let mut exclude_dirs = Vec::new();
    exclude_dirs.extend(file_config.filters.exclude_dirs);
    exclude_dirs.extend(options.exclude_dirs.clone());
    let mut exclude_extensions = Vec::new();
    exclude_extensions.extend(file_config.filters.exclude_extensions);
    exclude_extensions.extend(options.exclude_extensions.clone());
    Ok(EffectiveConfig {
        repo,
        since: Some("1 month ago".to_string()),
        until: None,
        branch: options.branch.clone(),
        no_merge: true,
        exclude_dirs,
        exclude_extensions,
    })
}

fn resolve_config_path(explicit: Option<&Path>, repo: &Path) -> Option<PathBuf> {
    if let Some(path) = explicit {
        return Some(path.to_path_buf());
    }
    let default = repo.join("git-report.toml");
    default.exists().then_some(default)
}

fn canonical_repo(repo: Option<&Path>) -> Result<PathBuf, String> {
    let path = repo.unwrap_or_else(|| Path::new("."));
    path.canonicalize()
        .map_err(|err| format!("failed to resolve repo {}: {err}", path.display()))
}

fn print_line(content: String) {
    println!("{content}");
}
