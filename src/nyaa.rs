use tui::text::Text;
use tui::style::{Style, Color};
use core::str::FromStr;
use urlencoding::encode;
use rss::{Channel, extension::Extension};
use std::error::Error;
use log::debug;
use std::collections::BTreeMap;
use num_derive::FromPrimitive;
use num;

pub mod config;

type ExtensionMap = BTreeMap<String, Vec<Extension>>;

pub struct Item {
    pub seeders: u32,
    pub leechers: u32,
    pub downloads: u32,
    pub title: String,
    pub torrent_link: String,
    pub category: Category,
    pub trusted: bool,
    pub remake: bool
}

impl Item {
    pub fn get_styled_title(&self) -> Text {
        if self.trusted {
            return Text::styled(self.title.to_owned(), Style::default().fg(Color::Green));
        } else if self.remake {
            return Text::styled(self.title.to_owned(), Style::default().fg(Color::Red));
        }
        Text::from(self.title.to_owned())
    }
}

#[derive(Copy, Clone)]
pub enum Filter {
    NoFilter = 0,
    NoRemakes = 1,
    TrustedOnly = 2
}

#[derive(Copy, Clone, FromPrimitive)]
pub enum Category {
    AllAnime = 0,
    AnimeMusicVideo = 1,
    EnglishTranslated = 2,
    NonEnglishTranslated = 3,
    Raw = 4
}

impl Category {
    fn get_url_string(&self) -> String {
        "1_".to_owned() + &(self.to_owned() as i32).to_string()
    }
    
    fn get_name(&self) -> String {
        match self {
            Category::AllAnime => "All Anime".to_owned(),
            Category::AnimeMusicVideo => "Anime Music Video".to_owned(),
            Category::EnglishTranslated => "English Translated".to_owned(),
            Category::NonEnglishTranslated => "Non-English Translated".to_owned(),
            Category::Raw => "Raw".to_owned()
        }
    }
    
    pub fn get_icon(&self) -> Text {
        match self {
            Category::AllAnime => Text::raw(""),
            Category::AnimeMusicVideo => Text::styled("AMV", Style::default().fg(Color::Magenta)),
            Category::EnglishTranslated => Text::styled("Subs", Style::default().fg(Color::Magenta)),
            Category::NonEnglishTranslated => Text::styled("Subs", Style::default().fg(Color::Green)),
            Category::Raw => Text::styled("Raw", Style::default().fg(Color::Gray))
        }
    }
}

impl Filter {
    fn get_url_string(&self) -> String {
        (self.to_owned() as i32).to_string()
    }
    
    fn get_name(self) -> String {
        match self {
            Filter::NoFilter => "No Filter".to_owned(),
            Filter::NoRemakes => "No Remakes".to_owned(),
            Filter::TrustedOnly => "TrustedOnly".to_owned()
        }
    }
}

pub async fn get_feed_list(query: &String, cat: &Category, filter: &Filter) -> Vec<Item> {
    let feed = match get_feed(query.to_owned(), cat, filter).await {
        Ok(x) => x,
        Err(_) => panic!("Failed to connect to nyaa.si...")
    };
    let mut items: Vec<Item> = Vec::new();
    
    for item in &feed.items {
        if let (Some(ext_map), Some(title), Some(link)) = (item.extensions().get("nyaa"), &item.title, &item.link) {
            let seeders = get_ext_value::<u32>(ext_map, "seeders").await.unwrap_or_default();
            let leechers = get_ext_value(ext_map, "leechers").await.unwrap_or_default();
            let downloads = get_ext_value(ext_map, "downloads").await.unwrap_or_default();
            let category_str: String = get_ext_value::<String>(ext_map, "categoryId").await.unwrap_or_default();
            let trusted: bool = get_ext_value::<String>(ext_map, "trusted").await.unwrap_or_default().eq("Yes");
            let remake: bool = get_ext_value::<String>(ext_map, "remake").await.unwrap_or_default().eq("Yes");
            
            items.push(Item {
                seeders,
                leechers,
                downloads,
                title: title.to_owned(),
                torrent_link: link.to_owned(),
                category: num::FromPrimitive::from_u32(category_str.chars().last().unwrap() as u32 - '0' as u32).unwrap(),
                trusted,
                remake
            });
        }
    }
    items
}

pub async fn get_feed(query: String, cat: &Category, filter: &Filter) -> Result<Channel, Box<dyn Error>> {
    let encoded_url = format!("https://nyaa.si/?page=rss&f={}&c={}&q={}&m", filter.get_url_string(), cat.get_url_string(), encode(&query));
    let content = reqwest::get(encoded_url)
        .await?
        .bytes()
        .await?;
    
    
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

pub async fn get_ext_value<T>(ext_map: &ExtensionMap, key: &str) -> Option<T> where T: FromStr {
    if let Some(seeders) = ext_map.get(key) {
        if let Some(seeders2) = seeders.get(0) {
            if let Some(seeder_value) = &seeders2.value {
                return match seeder_value.to_string().parse::<T>() {
                    Ok(x) => Some(x),
                    Err(_) => None
                }
            }
        }
    }
    None
}
