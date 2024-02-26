use std::collections::HashMap;

use crate::Rss;

pub struct RssManager {
    rss_feeds: HashMap<String, Rss>,
}

impl RssManager {
    pub fn new() -> Self {
        RssManager {
            rss_feeds: HashMap::new(),
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
