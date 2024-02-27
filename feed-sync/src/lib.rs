pub mod parser;

use feed_rs::model::{Content, Entry, Feed};
use reqwest::get;
use rusqlite::{params, Connection};
use std::collections::HashSet;

// Define your structs here

type IsLiked = bool;
#[derive(Debug, Clone)]
pub struct FeedManager {
    pub feeds: HashSet<(Feed, String)>,
    pub to_see: Vec<Entry>,
    pub already_seen: Vec<(Entry, IsLiked)>,
}
impl FeedManager {
    pub fn new() -> Self {
        FeedManager {
            feeds: HashSet::new(),
            to_see: Vec::new(),
            already_seen: Vec::new(),
        }
    }

    pub async fn new_feed(&mut self, url: &str) -> Result<Feed, Box<dyn std::error::Error>> {
        let xml = get(url).await?.text().await?;
        let feed = feed_rs::parser::parse(xml.as_bytes())?;
        self.add_feed(feed.clone(), url.to_string());
        self.sync().await;
        Ok(feed)
    }

    pub async fn sync(&mut self) {
        let mut new_feeds = HashSet::new();
        for (_, url) in self.feeds.iter() {
            let xml = get(url).await.unwrap().text().await.unwrap();
            let new_feed = feed_rs::parser::parse(xml.as_bytes()).unwrap();
            for entry in &new_feed.entries {
                if !self.to_see.contains(&entry) {
                    self.to_see.push(entry.clone());
                }
            }
            new_feeds.insert((new_feed, url.clone()));
        }
        self.feeds = new_feeds;
    }

    fn add_feed(&mut self, feed: Feed, url: String) {
        self.feeds.insert((feed, url));
    }

    pub fn remove_feed_by_url(&mut self, url: &str) {
        self.feeds.retain(|(_, u)| u != url);
    }
    pub fn get_feed(&self, url: &str) -> Option<&Feed> {
        self.feeds.iter().find(|(_, u)| u == url).map(|(f, _)| f)
    }

    pub fn save_already_seen(&self, db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Establish connection to the SQLite database
        let conn = Connection::open(db_path)?;

        // Create the necessary table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS already_seen (
                id TEXT PRIMARY KEY,
                title TEXT,
                authors TEXT,
                content TEXT,
                links TEXT,
                summary TEXT,
                categories TEXT,
                language TEXT,
                is_liked INTEGER
            )",
            [],
        )?;

        // Iterate over each already_seen entry and insert into the database
        for (entry, is_liked) in &self.already_seen {
            conn.execute(
                "INSERT OR REPLACE INTO already_seen (
                    id, title, authors, content, links, summary,
                    categories, language, is_liked
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    entry.id,
                    entry.title.as_ref().map_or("", |t| t.content.as_str()),
                    serde_json::to_string(
                        &entry
                            .authors
                            .iter()
                            .map(|a| a.name.as_str())
                            .collect::<String>()
                    )
                    .unwrap(),
                    serde_json::to_string(
                        &entry
                            .content
                            .clone()
                            .unwrap_or(Content::default())
                            .body
                            .unwrap_or_default()
                    )
                    .unwrap(),
                    serde_json::to_string(&entry.links.get(0).unwrap().href).unwrap(),
                    entry.summary.clone().unwrap_or_default().content,
                    serde_json::to_string(
                        &entry
                            .categories
                            .iter()
                            .map(|c| c.term.as_str())
                            .collect::<String>()
                    )
                    .unwrap(),
                    entry.language.as_ref().unwrap_or(&"".to_string()),
                    if *is_liked { 1 } else { 0 }
                ],
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_feed() {
        let (_mock, mg) = _build_from_mock().await;
        let addr = _mock.host_with_port();
        let addr = &format!("http://{}", addr);
        let feed = mg.get_feed(&addr);

        assert!(feed.is_some());
    }

    #[tokio::test]
    async fn test_new_feed_not_duplicating() {
        let (_mock, mut mg) = _build_from_mock().await;
        // add the same feed again to see if it's not duplicated
        let addr = _mock.host_with_port();
        let addr = &format!("http://{}", addr);
        mg.new_feed(addr).await.unwrap();

        assert_eq!(mg.feeds.len(), 1);

        assert!(!mg.feeds.is_empty());
        assert!(!mg.to_see.is_empty());
    }

    #[tokio::test]
    async fn test_remove_feed_by_url() {
        let (_mock, mut mg) = _build_from_mock().await;
        let addr = _mock.host_with_port();
        let addr = &format!("http://{}", addr);

        assert!(!mg.feeds.is_empty());
        mg.remove_feed_by_url(addr);
        assert!(mg.feeds.is_empty());
    }

    pub async fn _build_from_mock() -> (mockito::ServerGuard, FeedManager) {
        let mut _m = mockito::Server::new_async().await;
        _m.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/rss+xml")
            .with_body(_RSS)
            .create();
        let addr = _m.host_with_port();

        let mut mg = FeedManager::new();
        let _ = mg.new_feed(&format!("http://{}", addr)).await;

        (_m, mg)
    }

    const _RSS: &'static str = include_str!("../mocks/rss.xml");
}
