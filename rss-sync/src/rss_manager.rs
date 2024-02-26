use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use rss::Item;
use rusqlite::Connection;

use crate::Rss;

#[derive(Debug)]
pub struct RssManager {
    pub rss_feeds: HashMap<String, Rss>,
    pub all_news: Vec<Item>,
}

impl RssManager {
    pub fn new() -> Self {
        RssManager {
            rss_feeds: HashMap::new(),
            all_news: Vec::new(),
        }
    }

    pub fn add_feed(&mut self, name: String, rss: Rss) {
        self.rss_feeds.insert(name, rss);
    }

    pub async fn sync_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (_, rss) in &mut self.rss_feeds {
            rss.sync().await?;
        }
        Ok(())
    }

    pub fn get_feed(&self, name: &str) -> Option<&Rss> {
        self.rss_feeds.get(name)
    }

    pub fn get_feed_mut(&mut self, name: &str) -> Option<&mut Rss> {
        self.rss_feeds.get_mut(name)
    }

    pub fn update_all_news(&mut self) {
        let all_news = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        for rss in self.rss_feeds.values_mut() {
            let all_news_clone = Arc::clone(&all_news);
            let rss_clone = rss.clone();
            let handle = thread::spawn(move || {
                for item in &rss_clone.items {
                    all_news_clone.lock().unwrap().push(item.clone());
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
        self.all_news = all_news.lock().unwrap().clone();
    }

    pub fn save_to_database(&self, db_path: &str) -> Result<(), rusqlite::Error> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rss_items (
                id INTEGER PRIMARY KEY,
                title TEXT,
                link TEXT,
                description TEXT,
                author TEXT,
                categories TEXT,
                comments TEXT,
                pub_date TEXT,
                source TEXT,
                content TEXT,
                itunes_ext TEXT,
                dublin_core_ext TEXT,
                last_updated TEXT
            )",
            [],
        )?;

        for (_, rss) in &self.rss_feeds {
            for item in &rss.items {
                conn.execute(
                    "INSERT INTO rss_items (title, link, description, author, categories, comments, pub_date, source, content, itunes_ext, dublin_core_ext, last_updated)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    &[&item.title, &item.link, &item.description, &item.author, &format!("{:?}", item.categories).into(), &item.comments, &item.pub_date, &item.source.clone().unwrap_or_default().url.to_string().into()
                    , &item.content, &format!("{:?}", item.itunes_ext).into(), &format!("{:?}", item.dublin_core_ext).into(), &rss.last_updated.clone().into()],
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests;

    #[tokio::test]
    async fn test_add_feed_and_get_feed() {
        let mut rss_manager = RssManager::new();
        let (_, rss) = tests::_build_from_tempfile();
        rss_manager.add_feed("SampleFeed".to_string(), rss.clone());

        // Check if feed is added correctly
        assert_eq!(rss_manager.get_feed("SampleFeed"), Some(&rss));
    }

    #[tokio::test]
    async fn test_sync_all() {
        let mut rss_manager = RssManager::new();

        let (_f1, rss1) = tests::_build_from_tempfile(); // capture file to avoid dropping it
        let (_f2, rss2) = tests::_build_from_tempfile();

        rss_manager.add_feed("SampleFeed1".to_string(), rss1);
        rss_manager.add_feed("SampleFeed2".to_string(), rss2);

        // Synchronize all feeds
        rss_manager.sync_all().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_feed_mut() {
        let mut rss_manager = RssManager::new();
        let (_, rss) = tests::_build_from_tempfile();
        rss_manager.add_feed("SampleFeed".to_string(), rss.clone());

        if let Some(feed) = rss_manager.get_feed_mut("SampleFeed") {
            feed.last_updated = "2024-02-25".to_string();
        }

        assert_eq!(
            rss_manager
                .get_feed("SampleFeed")
                .map(|feed| &feed.last_updated),
            Some(&"2024-02-25".to_string())
        );
    }
}
