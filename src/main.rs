use std::path::PathBuf;

use chrono::{Duration, Utc};
use structsy::{Structsy, StructsyTx};
use tfs::{Error, Item};
use titan_fitness_stock as tfs;

use clap::Parser;

#[derive(Parser, Debug)]
enum Opt {
    /// Check for new merchandise, storing the item data and timestamp for this check
    Run(RunArgs),
    /// Lookup all known items in the provided database that have been seen in the last 30 days
    CurrentAsCsv(CsvArgs),
}

#[derive(Parser, Debug)]
struct RunArgs {
    #[arg(long = "db", short)]
    /// The path to a database file. On first run, this will establish
    /// your database so when it is run again, it can compare the current
    /// available stock.
    db_path: PathBuf,
    #[arg(long)]
    /// The path to a directory you would like time stamped html written into
    /// when new items are found
    debug_html: Option<PathBuf>,
}
#[derive(Parser, Debug)]
struct CsvArgs {
    #[arg(long, short)]
    /// A path to an output file, if not provided csv will be printed
    /// to stdout
    out: Option<String>,
    #[arg(long = "db", short)]
    /// The path to a previously created database file. Execute the `run` command
    /// to create one
    db_path: PathBuf,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::try_init().ok();
    let opt = Opt::parse();
    let r = match opt {
        Opt::Run(args) => daily_check(args).await,
        Opt::CurrentAsCsv(args) => current_as_csv(args),
    };
    match r {
        Err(Error::Reqwest(_)) => {}
        Err(e) => println!("{}", e),
        _ => {}
    }
}

async fn daily_check(args: RunArgs) -> Result<(), Error> {
    tfs::daily_new_item_check(&args.db_path, &args.debug_html).await?;
    Ok(())
}

fn current_as_csv(args: CsvArgs) -> Result<(), Error> {
    let db = tfs::open_db(&args.db_path)?;

    if let Some(path) = &args.out {
        let f = std::fs::File::create(path).map_err(|e| {
            log::error!("Failed to create file at {}: {}", path, e);
            e
        })?;
        write_csv(csv::WriterBuilder::new().from_writer(f), &db).map_err(|e| {
            log::error!("Failed to write CSV to {}: {}", path, e);
            e
        })?;
    } else {
        let stdout = std::io::stdout();
        write_csv(csv::WriterBuilder::new().from_writer(stdout), &db).map_err(|e| {
            log::error!("Failed to write CSV to stdout: {}", e);
            e
        })?;
    }
    Ok(())
}

fn write_csv<T: std::io::Write>(mut w: csv::Writer<T>, db: &Structsy) -> Result<(), Error> {
    use tfs::ItemQueries;
    let now = Utc::now();
    let prev = now - Duration::days(30);
    let start = prev.timestamp() as u64;
    let end = now.timestamp() as u64;
    let mut tx = db.begin()?;
    let total_items = tx.scan::<Item>()?.count();
    log::debug!(
        "looking up items between {} and {}. total: {}",
        now,
        prev,
        total_items
    );
    for (_id, item) in db.query::<Item>().in_timestamp_range(start..end) {
        w.serialize(&item).map_err(|e| {
            log::error!("Failed to write item as csv {}: {}", item.name, e);
            e
        })?;
    }
    w.flush()?;
    Ok(())
}
