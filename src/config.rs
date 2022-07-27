use std::str::FromStr;

use argh::FromArgs;
use serde::Deserialize;

/// a RSS channel retriever
#[derive(Debug, Deserialize, FromArgs)]
pub struct Config {
    /// display ordering for RSS items [date | channel]
    #[argh(option, default = "Order::Date")]
    pub display_by: Order,

    /// a list of RSS feed urls
    #[argh(positional)]
    pub channels: Vec<ChanConfig>,
}

impl Config {
    /// Validate if this Config is usable
    pub fn validate(&self) -> Result<(), &str> {
        if self.channels.is_empty() {
            return Err("No channels configured.");
        }
        Ok(())
    }

    /// Override Self with `other`'s fields
    pub fn override_with(self, other: Config) -> Config {
        let channels = if other.channels.is_empty() {
            self.channels
        } else {
            other.channels
        };
        Config {
            channels,
            display_by: other.display_by,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Order {
    /// Order by item date
    Date,
    /// Order by item's channel
    Channel,
}

impl FromStr for Order {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "date" => Ok(Order::Date),
            "channel" => Ok(Order::Channel),
            _ => Err("Unrecognized Order. [date | channel]".to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChanConfig {
    pub url: String,
    #[serde(default)]
    pub max_items: Option<usize>,
    pub item_config: Option<ItemConfig>,
}

impl FromStr for ChanConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ChanConfig {
            url: s.to_owned(),
            max_items: None,
            item_config: Some(Default::default()),
        })
    }
}

/// Toggles for displaying Item fields
#[derive(Debug, Default, Deserialize, Clone, Copy)]
pub struct ItemConfig {
    #[serde(default)]
    pub hide_title: bool,
    #[serde(default)]
    pub hide_link: bool,
    #[serde(default)]
    pub hide_description: bool,
    #[serde(default)]
    pub hide_author: bool,
    #[serde(default)]
    pub hide_pub_date: bool,
    #[serde(default)]
    pub show_enclosure: bool,
}
