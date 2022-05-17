use anyhow::Result;
use clap::Parser;
use walkdir::WalkDir;
use warm_fs::Warmer;

const SECS_IN_DAY: u64 = 60 * 60 * 24;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of threads
    #[clap(short, long, default_value_t = 100)]
    threads: usize,

    /// Chunk size, `warm_fs` leaks memory without it
    #[clap(short, long, default_value_t = 1000)]
    chunk: usize,

    /// Infrequent access porlicy archivation period in days
    #[clap(short, long, default_value_t = 30)]
    ia_days: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let paths = get_paths(args.ia_days)?;
    let mut warmer = Warmer::new(args.threads, false);
    warmer.add_dirs(&paths);
    let bar = progress_bar(0);

    bar.set_prefix("Size estimation");
    for n in warmer.iter_estimate() {
        bar.inc_length(n);
    }

    bar.set_prefix("Files reading");
    for chunk in paths.as_slice().chunks(args.chunk) {
        let mut warmer = Warmer::new(args.threads, false);
        warmer.add_dirs(&chunk);

        for n in warmer.iter_warm() {
            bar.inc(n);
        }
        bar.println("Chunk is done");
    }

    bar.abandon();
    Ok(())
}

fn progress_bar(total: u64) -> indicatif::ProgressBar {
    let bar = indicatif::ProgressBar::new(total);
    bar.set_style(indicatif::ProgressStyle::default_bar().template(
        "{prefix} {bar} {bytes} of {total_bytes} {percent}% {binary_bytes_per_sec} ~{eta} {msg}",
    ));
    bar.set_draw_rate(25);
    bar
}

/// Gets paths starting from the current dir created before `elapsed_days` days
fn get_paths(elapsed_days: u64) -> Result<Vec<std::path::PathBuf>> {
    let elapsed_secs = SECS_IN_DAY * elapsed_days;
    let mut paths = Vec::new();
    for entry in WalkDir::new(".")
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let secs = entry.metadata()?.modified()?.elapsed()?.as_secs();
        if secs < elapsed_secs {
            continue;
        }
        paths.push(path.into())
    }
    Ok(paths)
}
