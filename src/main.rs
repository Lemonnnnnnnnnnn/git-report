fn main() {
    if let Err(err) = git_report::cli::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
