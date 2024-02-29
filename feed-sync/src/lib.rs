pub mod parser;
pub mod persistence;

use feed_rs::model::{Entry, Feed};
use reqwest::get;
use tokio::task;

use std::collections::HashSet;

// Define your structs here

type IsLiked = bool;
#[derive(Debug, Clone)]
pub struct FeedManager {
    pub feeds: HashSet<(Feed, String)>,
    pub to_see: Vec<Entry>,
    pub already_seen: Vec<(Entry, IsLiked)>,
}
unsafe impl Send for FeedManager {}

impl FeedManager {
    pub fn new() -> Self {
        FeedManager {
            feeds: HashSet::new(),
            to_see: Vec::new(),
            already_seen: Vec::new(),
        }
    }

    pub async fn new_feed(&mut self, url: &str) -> Result<Feed, Box<dyn std::error::Error>> {
        self.add_feed(default_feed(), url.to_string());
        self.sync().await;
        let feed = self.get_feed(url).unwrap().clone();
        Ok(feed)
    }

    pub async fn sync(&mut self) {
        let mut new_feeds = HashSet::new();
        self.to_see.clear();

        let mut tasks = Vec::new();

        for (_, url) in self.feeds.iter() {
            let url = url.clone();

            let task = task::spawn(async move {
                let xml = get(&url).await.unwrap().text().await.unwrap();
                let new_feed = feed_rs::parser::parse(xml.as_bytes()).unwrap();
                (new_feed, url)
            });

            tasks.push(task);
        }

        for task in tasks {
            let (new_feed, url) = task.await.unwrap();
            for entry in &new_feed.entries {
                if !self.to_see.contains(&entry) {
                    self.to_see.push(entry.clone());
                }
            }
            new_feeds.insert((new_feed, url));
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

fn default_feed() -> Feed {
    Feed {
        id: "".to_string(),
        title: None,
        updated: None,
        authors: vec![],
        links: vec![],
        categories: vec![],
        contributors: vec![],
        generator: None,
        icon: None,
        logo: None,
        rights: None,
        entries: vec![],
        language: None,
        feed_type: feed_rs::model::FeedType::Atom,
        description: None,
        published: None,
        rating: None,
        ttl: None,
    }
}
