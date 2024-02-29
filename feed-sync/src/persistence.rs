use feed_rs::model::{Content, Link};
use rusqlite::{params, Connection};

use crate::{default_feed, FeedManager};

impl FeedManager {
    // persistence
    pub fn save_already_seen(&self, db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;

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

        let mut stmt = conn.prepare(
            "
          INSERT OR REPLACE INTO already_seen (
              id, title, authors, content, links, summary,
              categories, language, is_liked
          ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
      ",
        )?;

        Self::execute_each_entry_in_already_seen(self, &mut stmt)
    }

    fn execute_each_entry_in_already_seen(
        &self,
        stmt: &mut rusqlite::Statement,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (entry, is_liked) in &self.already_seen {
            let authors_json = serde_json::to_string(
                &entry
                    .authors
                    .iter()
                    .map(|a| a.name.as_str().to_lowercase())
                    .collect::<Vec<_>>(),
            )?;
            let content_json = serde_json::to_string(
                &entry
                    .content
                    .clone()
                    .unwrap_or(Content::default())
                    .body
                    .unwrap_or_default()
                    .to_lowercase(),
            )?;
            let links_json = serde_json::to_string(
                &entry
                    .links
                    .get(0)
                    .unwrap_or(&Link {
                        href: String::new(),
                        rel: None,
                        media_type: None,
                        href_lang: None,
                        title: None,
                        length: None,
                    })
                    .href
                    .to_lowercase(),
            )?;
            let categories_json = serde_json::to_string(
                &entry
                    .categories
                    .iter()
                    .map(|c| c.term.as_str().to_lowercase())
                    .collect::<Vec<_>>(),
            )?;
            let language = entry
                .language
                .as_ref()
                .map_or("", String::as_str)
                .to_lowercase();

            let title = entry
                .title
                .as_ref()
                .map_or("", |t| t.content.as_str())
                .to_lowercase();

            stmt.execute(params![
                entry.id.to_lowercase(),
                title,
                authors_json,
                content_json,
                links_json,
                entry
                    .summary
                    .clone()
                    .unwrap_or_default()
                    .content
                    .to_lowercase(),
                categories_json,
                language,
                if *is_liked { 1 } else { 0 }
            ])?;
        }
        Ok(())
    }

    pub fn load_feeds_from_db(&mut self, db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS feeds (
              id TEXT PRIMARY KEY,
              url TEXT
          )",
            [],
        )?;
        let mut stmt = conn.prepare("SELECT url FROM feeds")?;
        let feeds = stmt.query_map([], |row| Ok(row.get(0)?))?;
        for feed in feeds {
            let url: String = feed?;
            self.add_feed(default_feed(), url)
        }
        Ok(())
    }

    pub fn save_feeds(&self, db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS feeds (
              id TEXT PRIMARY KEY,
              url TEXT
          )",
            [],
        )?;

        let mut stmt = conn.prepare(
            "
          INSERT OR REPLACE INTO feeds (
              id, url
          ) VALUES (?1, ?2)
      ",
        )?;

        for (feed, url) in &self.feeds {
            let id = feed.id.clone();
            stmt.execute(params![id, url])?;
        }
        Ok(())
    }

    pub fn purge_feed(
        &mut self,
        db_path: &str,
        url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS feeds (
              id TEXT PRIMARY KEY,
              url TEXT
          )",
            [],
        )?;

        let mut stmt = conn.prepare("DELETE FROM feeds WHERE url = ?1")?;
        stmt.execute(params![url])?;
        self.remove_feed_by_url(url);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use feed_rs::model::Entry;

    use crate::tests::_build_from_mock;

    use super::*;

    use std::path::Path;

    #[test]
    fn test_save_already_seen() {
        let binding = random_db_path();
        let db_path = binding.as_str();
        let mut manager = FeedManager::new();
        manager.already_seen.push((Entry::default(), false));
        let _ = manager
            .save_already_seen(db_path)
            .expect("Failed to save already seen entries to the database");

        assert!(Path::new(db_path).exists());
        let conn = Connection::open(db_path).expect("Failed to open test file");
        let mut stmt = conn
            .prepare("SELECT * FROM already_seen")
            .expect("Failed to prepare statement");
        let entries = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, i64>(8)?,
                ))
            })
            .expect("Failed to query map");

        for entry in entries {
            let (id, title, authors, content, links, summary, categories, language, is_liked) =
                entry.expect("Failed to get entry");

            assert_eq!(id, "");
            assert_eq!(title, "");
            assert_eq!(authors, "[]");
            assert_eq!(content, "\"\"");
            assert_eq!(links, "\"\"");
            assert_eq!(summary, "");
            assert_eq!(categories, "[]");
            assert_eq!(language, "");
            assert_eq!(is_liked, 0);
        }
        std::fs::remove_file(db_path).expect("Failed to remove test file");
    }

    #[tokio::test]
    async fn test_save_feeds() {
        let binding = random_db_path();
        let db_path = binding.as_str();
        std::fs::File::create(db_path).expect("Failed to create test file");
        let (_mock, manager) = _build_from_mock().await;

        let _ = manager
            .save_feeds(db_path)
            .expect("Failed to save feeds to the database");

        let mut copy_manager = FeedManager::new();
        copy_manager.load_feeds_from_db(db_path).unwrap();
        copy_manager.sync().await;

        assert!(Path::new(db_path).exists());
        assert_eq!(manager.feeds, copy_manager.feeds);
        std::fs::remove_file(db_path).expect("Failed to remove test file");
    }

    #[tokio::test]
    async fn test_purge_feed() {
        let binding = random_db_path();
        let db_path = binding.as_str();
        std::fs::File::create(db_path).expect("Failed to create test file");
        let (_mock, mut manager) = _build_from_mock().await;

        let binding = _mock.url();
        let url = binding.as_str();
        let feed = manager.get_feed(url).unwrap().clone();

        let _ = manager
            .purge_feed(db_path, url)
            .expect("Failed to purge feed from the database");

        assert!(!manager.feeds.contains(&(feed, url.to_string())));
        std::fs::remove_file(db_path).expect("Failed to remove test file");
    }
    fn random_db_path() -> String {
        (DB_FOLDER.to_owned() + &uuid::Uuid::new_v4().to_string())
            .as_str()
            .to_owned()
    }
    const DB_FOLDER: &str = "../db/";
}
