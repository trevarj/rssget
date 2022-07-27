#![feature(iterator_try_collect)]
#![feature(once_cell)]
use std::error::Error;
use std::fmt::Write;
use std::fs::OpenOptions;
use std::io::{BufReader, ErrorKind};
use std::sync::LazyLock;

use chrono::{DateTime, FixedOffset};
use rss::{Channel, Item};
use textwrap::Options;

use crate::config::{Config, ItemConfig, Order};

mod config;

static WRAP_OPTIONS: LazyLock<Options> = LazyLock::new(|| {
    Options::new(80)
        .initial_indent("     ")
        .subsequent_indent("     ")
});

#[derive(Debug)]
struct DisplayItem {
    /// Channel name/title
    pub chan_title: String,
    /// Item config
    pub conf: ItemConfig,
    /// The title of the item.
    pub title: Option<String>,
    /// The URL of the item.
    pub link: Option<String>,
    /// The item synopsis.
    pub description: Option<String>,
    /// The email address of author of the item.
    pub author: Option<String>,
    /// The date the item was published as an RFC 2822 timestamp.
    pub pub_date: Option<DateTime<FixedOffset>>,
}

impl DisplayItem {
    /// Create a new DisplayItem from RSS Item
    pub fn new(item: Item, chan_title: &str, conf: &ItemConfig) -> DisplayItem {
        DisplayItem {
            chan_title: chan_title.to_string(),
            conf: conf.to_owned(),
            title: item.title,
            link: item.link,
            description: item.description,
            author: item.author,
            pub_date: item
                .pub_date
                .and_then(|d| DateTime::<FixedOffset>::parse_from_rfc2822(&d).ok()),
        }
    }

    /// Build formatted RSS Item
    pub fn format(&self) -> Result<String, std::fmt::Error> {
        let mut out = String::new();
        if let Some(pub_date) = self.pub_date && !self.conf.hide_pub_date {
            write!(out, "[{}] - ", pub_date.naive_local())?;
        }
        writeln!(out, "{}", self.chan_title)?;
        if let Some(title) = &self.title && !self.conf.hide_title {
            writeln!(out, "{}", textwrap::fill(title, &*WRAP_OPTIONS))?;
        }
        if let Some(author) = &self.author && !self.conf.hide_author {
            writeln!(out, " - {}", author)?;
        }
        if let Some(desc) = &self.description && !self.conf.hide_description {
            writeln!(out, "{}", textwrap::fill(desc, &*WRAP_OPTIONS))?;
        }
        if let Some(link) = &self.link && !self.conf.hide_link {
            write!(out, "[{}]", link)?;
        }
        Ok(out)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // get cli flags
    let args: Config = argh::from_env();

    // open config file if present
    let config = dirs::config_dir()
        .map(|mut d| {
            d.push("rssget/config.yaml");
            d
        })
        .expect("could not determine system config directory");
    let config = match OpenOptions::new().read(true).open(&config) {
        Ok(file) => {
            // parse config file
            let config_file: Config = serde_yaml::from_reader(&file)?;
            // override with any flags provided
            config_file.override_with(args)
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            // config file not found, try to use flags
            args
        }
        Err(err) => return Err(Box::new(err)),
    };

    config.validate()?;

    // call out to all rss feeds
    let http = ureq::agent();

    let mut items = config
        .channels
        .iter()
        .flat_map(|conf| match http.get(&conf.url).call() {
            Ok(res) => match Channel::read_from(BufReader::new(res.into_reader())) {
                Ok(chan) => chan
                    .items
                    .into_iter()
                    .take(conf.max_items.unwrap_or(usize::MAX))
                    .map(|item| {
                        DisplayItem::new(item, &chan.title, &conf.item_config.unwrap_or_default())
                    })
                    .collect(),
                Err(err) => {
                    eprintln!(
                        "Could not parse rss response from chan {}: {}",
                        conf.url, err
                    );
                    vec![]
                }
            },
            Err(err) => {
                eprintln!("Could not reach {}: {}", conf.url, err);
                vec![]
            }
        })
        .collect::<Vec<DisplayItem>>();

    if items.is_empty() {
        eprintln!("No RSS items found.");
        return Ok(());
    }

    items.sort_by_key(|i| match config.display_by {
        Order::Date => i.pub_date,
        Order::Channel => None,
    });

    items.iter().for_each(|i| match i.format() {
        Ok(output) => println!("{}", output),
        Err(err) => eprintln!("Could not format RSS Item: {}", err),
    });
    Ok(())
}
