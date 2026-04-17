# git-report

A reusable Git activity reporting CLI written in Rust.

It can summarize commit activity for a repository, aggregate stats by author, and render Markdown/JSON reports. The tool calls the system `git` binary, so it works best in environments where `git` is already installed.

## Build

```bash
cargo build --release
```

Binary output:

```bash
./target/release/git-report
```

Run the local dashboard:

```bash
./target/release/git-report web --repo /path/to/repo --open
```

## Commands

### `summary`

Print repository-wide totals for the selected range.

```bash
git-report summary --repo /path/to/repo --since "1 month ago"
git-report summary --repo /path/to/repo --format json
```

Supported options:

- `--repo <path>`: target Git repository, default is current directory
- `--since <expr>`: git-compatible start time, for example `1 month ago`
- `--until <expr>`: git-compatible end time
- `--branch <name>`: optional branch/revision to query
- `--config <path>`: TOML config file
- `--no-merge`: skip merge commits entirely
- `--exclude-dir <path-fragment>`: exclude matching paths
- `--exclude-ext <extension>`: exclude matching file extensions
- `--format <table|json|markdown>`: output format for stdout

### `authors`

Print stats grouped by author.

```bash
git-report authors --repo /path/to/repo
git-report authors --repo /path/to/repo --format json
```

The available options are the same as `summary`.

### `report`

Render a full report to stdout or a file.

```bash
git-report report --repo /path/to/repo --since "30 days ago" --format markdown
git-report report --repo /path/to/repo --format json --output report.json
```

Additional option:

- `--output <path>`: write the rendered report to a file instead of stdout

### `preset monthly-authors`

Run the built-in preset that replaces the original monthly report script.

Default behavior:

- time range: last month
- merge commits: excluded

Example:

```bash
git-report preset monthly-authors --repo /path/to/repo --output-dir ./git-reports
```

This writes:

- `monthly-report.json`
- `monthly-report.md`

Supported options:

- `--repo <path>`
- `--branch <name>`
- `--config <path>`
- `--output-dir <path>`
- `--exclude-dir <path-fragment>`
- `--exclude-ext <extension>`

### `web`

Start a local web server and open an interactive dashboard for the selected repository.

```bash
git-report web --repo /path/to/repo --open
git-report web --repo /path/to/repo --since "90 days ago" --port 4123
```

Supported options:

- `--repo <path>`
- `--since <expr>`
- `--until <expr>`
- `--branch <name>`
- `--config <path>`
- `--no-merge`
- `--exclude-dir <path-fragment>`
- `--exclude-ext <extension>`
- `--host <host>`: local bind host, default `127.0.0.1`
- `--port <port>`: local bind port, default `3000`
- `--open`: open the dashboard in the default browser

The dashboard includes:

- KPI summary cards
- commit and line-change trends
- contributor charts
- author detail table
- hot path table
- interactive local filtering

## Config File

If `git-report.toml` exists in the target repository, it is loaded automatically. You can also pass a config path explicitly with `--config`.

Example:

```toml
[filters]
exclude_dirs = ["apps/operation/src/services/"]
exclude_extensions = [".generated.ts"]
```

Command-line filters are appended to the config-defined filters.

## Output Notes

- `summary --format json` prints the summary object only
- `authors --format json` prints the author array only
- `report --format json` prints the full structured report
- binary file changes are ignored in line-count totals
- path filtering applies before additions/deletions are aggregated

## Typical Usage

Generate a quick author leaderboard:

```bash
git-report authors --repo /path/to/repo --since "2 weeks ago"
```

Generate a JSON report for automation:

```bash
git-report report --repo /path/to/repo --since "2026-04-01" --format json --output report.json
```

Generate a Markdown monthly report compatible with the old workflow:

```bash
git-report preset monthly-authors --repo /path/to/repo --output-dir docs/git-reports
```

Launch the interactive dashboard:

```bash
git-report web --repo /path/to/repo --open
```

## Frontend Development

The dashboard frontend lives in `web/` and uses:

- React
- Vite
- Tailwind CSS
- shadcn-style UI components
- Recharts

Install and rebuild frontend assets:

```bash
cd web
npm install
npm run build
```

The Rust server embeds files from `web/dist` at compile time, so rebuild the frontend before shipping a new binary if the UI changed.
