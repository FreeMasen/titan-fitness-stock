use std::path::PathBuf;

use chrono::{Duration, Utc};
use structsy::{Structsy, StructsyTx};
use tfs::{Error, Item};
use titan_fitness_stock as tfs;

use structopt::{clap::AppSettings, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(setting = AppSettings::InferSubcommands)]
enum Opt {
    Run(RunArgs),
    CurrentAsCsv(CsvArgs),
}

#[derive(StructOpt, Debug)]
struct RunArgs {
    #[structopt(long = "db", short = "d")]
    db_path: PathBuf,
}
#[derive(StructOpt, Debug)]
struct CsvArgs {
    #[structopt(long, short)]
    out: Option<String>,
    #[structopt(long = "db", short = "d")]
    db_path: PathBuf,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::try_init().ok();
    let opt = Opt::from_args();
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
    tfs::daily_new_item_check(&args.db_path).await?;
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
    log::debug!("looking up items between {} and {}. total: {}", now, prev, total_items);
    for (_id, item) in db.query::<Item>().in_timestamp_range(start..end) {
        w.serialize(&item).map_err(|e| {
            log::error!("Failed to write item as csv {}: {}", item.name, e);
            e
        })?;
    }
    w.flush()?;
    Ok(())
}
