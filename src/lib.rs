use std::ops::RangeBounds;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use structsy::{Structsy, StructsyTx};
use structsy_derive::{queries, Persistent, PersistentEmbedded};

use chrono::{Duration, Local, Utc};
use serde::{Deserialize, Serialize};

static URL: &str = "https://www.titan.fitness/on/demandware.store/Sites-TitanFitness-Site/default/Search-UpdateGrid?cgid=in-stock-items&start=0&viewall=true";

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    Csv(csv::Error),
    Structsy(structsy::StructsyError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO failure: {}", e),
            Self::Reqwest(e) => write!(f, "Network failure: {}", e),
            Self::Serde(e) => write!(f, "JSON Data failure: {}", e),
            Self::Csv(e) => write!(f, "CSV Data failure: {}", e),
            Self::Structsy(e) => write!(f, "Structsy error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}
impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Self {
        Self::Csv(e)
    }
}

impl From<structsy::StructsyError> for Error {
    fn from(other: structsy::StructsyError) -> Self {
        Self::Structsy(other)
    }
}

pub async fn daily_new_item_check<P: AsRef<Path>>(
    db_path: P,
    debug_path: &Option<PathBuf>,
) -> Result<()> {
    let start = chrono::Local::now();
    let text = request_html().await?;

    let new_values = html_to_items(&text);
    let database = open_db(&db_path)?;
    let end_dt = Utc::now();
    let now = end_dt.timestamp() as u64;
    let mut printed_preamble = false;
    for (new_id, mut new_item) in new_values {
        new_item.last_seen = now;
        let mut q = database.query::<Item>().by_id(new_id).into_iter();
        if let Some((old_item_ref, old_item)) = q.next() {
            let diff = now - old_item.last_seen;
            if Duration::seconds(diff as _) > Duration::days(1) {
                if !printed_preamble {
                    printed_preamble = true;
                    println!("TFS Report for {}", start);
                    write_debug_html(&text, &debug_path, start)
                        .map_err(|e| eprintln!("error writing debug html {}", e))
                        .ok();
                }
                println!(
                    "new item: {}: {} ({})",
                    new_item.name, new_item.price, new_item.link
                );
            }
            let mut tx = database.begin()?;
            tx.update(&old_item_ref, &new_item)?;
            tx.commit()?;
        } else {
            if !printed_preamble {
                printed_preamble = true;
                println!("TFS Report for {}", start);
                write_debug_html(&text, &debug_path, start)
                    .map_err(|e| eprintln!("error writing debug html {}", e))
                    .ok();
            }
            println!(
                "new item: {}: {} ({})",
                new_item.name, new_item.price, new_item.link
            );
            let mut tx = database.begin()?;
            tx.insert(&new_item)?;
            tx.commit()?;
        }
    }

    Ok(())
}

fn write_debug_html(
    text: &str,
    debug_path: &Option<PathBuf>,
    start: chrono::DateTime<Local>,
) -> Result<()> {
    if let Some(dir) = debug_path.as_ref() {
        println!("Writing html");
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        let file_name = format!("{}.html", start);
        let full_path = dir.join(&file_name);
        std::fs::write(&full_path, &text)?;
        println!("html available at 'file://{}'", full_path.display());
    }
    Ok(())
}

pub async fn request_html() -> Result<String> {
    let mut ct = 0u8;
    loop {
        match reqwest::get(URL).await?.text().await {
            Ok(html) => break Ok(html),
            Err(e) => {
                if ct >= 5 {
                    log::error!("Failed to fetch html 5 times: {}", e);
                    break Err(e.into());
                }
            }
        }
        ct += 1;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

pub fn open_db<P: AsRef<Path>>(path: &P) -> Result<Structsy> {
    let db = Structsy::open(&path)?;
    db.define::<Item>()?;
    Ok(db)
}

use scraper::{Html, Selector};

pub fn html_to_items(text: &str) -> HashMap<String, Item> {
    const TOP_LEVEL: &str = ".product";
    const JSON: &str = ".gtmproduct";
    const LINK: &str = ".gtm-product-list.link";
    let fragment = Html::parse_fragment(text);
    let top_level_selector = Selector::parse(TOP_LEVEL).unwrap();
    let json_selector = Selector::parse(JSON).unwrap();
    let link_selector = Selector::parse(LINK).unwrap();
    fragment
        .select(&top_level_selector)
        .into_iter()
        .enumerate()
        .filter_map(|(i, ele)| {
            let (json, link) = find_item_parts(ele, &json_selector, &link_selector)?;
            let mut item: Item = serde_json::from_str(&json)
                .map_err(|e| {
                    eprintln!("Error parsing item {}: {}", i, e);
                    std::fs::write(&format!("item-{}.json", i), &json).unwrap();
                })
                .ok()?;
            item.link = link;
            Some((item.id.clone(), item))
        })
        .collect()
}

pub fn find_item_parts(
    product_frag: scraper::ElementRef,
    json_selector: &Selector,
    link_selector: &Selector,
) -> Option<(String, String)> {
    let json_el = product_frag.select(&json_selector).next()?;
    let json = json_el.value().attr("data-object")?.to_string();
    let link_el = product_frag.select(&link_selector).next()?;
    let link = link_el.value().attr("href")?;
    Some((json, format!("https://www.titan.fitness{}", link)))
}

#[queries(Item)]
pub trait ItemQueries {
    fn by_id(self, id: String) -> Self;
    fn in_timestamp_range<R: RangeBounds<u64>>(self, last_seen: R) -> Self;
}

#[derive(Debug, Deserialize, Serialize, Persistent)]
pub struct Item {
    #[index(mode = "cluster")]
    #[serde(default)]
    pub last_seen: u64,
    pub name: String,
    #[index(mode = "exclusive")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub price: Price,
    pub category: String,
    pub brand: String,
    pub position: String,
    pub list: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub back_in_stock: bool,
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    T: std::default::Default + Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Deserialize, Serialize, PersistentEmbedded)]
#[serde(untagged)]
pub enum Price {
    Single(f32),
    Range(String),
}

impl std::default::Default for Price {
    fn default() -> Self {
        Self::Single(0.0)
    }
}

impl std::fmt::Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Single(v) => write!(f, "{}", v),
            Self::Range(s) => write!(f, "{}", s),
        }
    }
}
