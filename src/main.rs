use anyhow::Result;
use clap::Parser;
use mailmap_linter::run_linter;

#[derive(Parser, Debug)]
#[command(name = "mailmap-linter")]
#[command(about = "Verify .mailmap file format and author mappings")]
struct Args {
    /// Regex patterns to exclude authors from verification
    #[arg(short, long)]
    exclude: Vec<String>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .compact()
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    if let Err(e) = run_linter(".mailmap", ".mailmap-exclude", args.exclude) {
        tracing::error!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
