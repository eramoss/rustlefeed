use feed_rs::model::Content;
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
            let links_json =
                serde_json::to_string(&entry.links.get(0).unwrap().href.to_lowercase())?;
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
