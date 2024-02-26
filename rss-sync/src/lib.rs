pub mod rss_manager;

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use rss::{Channel, Item};

#[derive(PartialEq, Clone, Debug)]
pub struct Rss {
    origin: Origin,
    pub channel: Channel,
    pub last_updated: String,
    pub items: Vec<Item>,
}

#[derive(PartialEq, Clone, Debug)]
enum Origin {
    Url(String),
    File(String),
}

impl Rss {
    pub async fn from_url(url: &str) -> Result<Rss, Box<dyn std::error::Error>> {
        let content = reqwest::get(url).await?.bytes().await?;
        Self::build(&content[..], Origin::Url(url.to_string()))
    }

    pub fn from_file(filepath: &str) -> Result<Rss, Box<dyn std::error::Error>> {
        let file = File::open(filepath).unwrap();
        Self::build(BufReader::new(file), Origin::File(filepath.to_string()))
    }

    pub async fn sync(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.origin {
            Origin::Url(url) => {
                let content = reqwest::get(url).await?.bytes().await?;
                let new_channel = Channel::read_from(&content[..])?;
                self.update(new_channel)?;
            }
            Origin::File(filepath) => {
                let file = File::open(filepath).unwrap();
                let new_channel = Channel::read_from(BufReader::new(file))?;
                self.update(new_channel)?;
            }
        }
        Ok(())
    }

    fn update(&mut self, new_channel: Channel) -> Result<(), Box<dyn std::error::Error>> {
        let last_updated = new_channel.last_build_date().unwrap().to_string();
        Ok(if last_updated != self.last_updated {
            self.sync_items(new_channel.items())?;
            self.last_updated = last_updated;
        })
    }

    fn sync_items(&mut self, new_items: &[Item]) -> Result<(), Box<dyn std::error::Error>> {
        let mut new_items = new_items.to_vec();
        new_items.retain(|new_item| !self.items.iter().any(|item| item.link() == new_item.link()));
        Ok(self.items.extend(new_items))
    }

    fn build<R>(content: R, origin: Origin) -> Result<Rss, Box<dyn std::error::Error>>
    where
        R: BufRead,
    {
        let channel = Channel::read_from(content)?;
        let last_updated = channel.last_build_date().unwrap().to_string();
        let items = channel.items();
        Ok(Rss {
            origin,
            channel: channel.clone(),
            last_updated,
            items: items.to_vec(),
        })
    }
}

#[cfg(test)]
pub mod tests {
    use tempfile::NamedTempFile;

    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_rss_from_url() {
        let (_m, rss) = _build_from_mock().await;
        let addr = _m.host_with_port();

        assert_eq!(rss.origin, Origin::Url(format!("http://{}", addr)));
        assert_eq!(rss.last_updated, _RSS_LAST_UPDATED);
    }

    #[test]
    fn test_rss_from_file() {
        let (file, rss) = _build_from_tempfile();

        assert_eq!(
            rss.origin,
            Origin::File(file.path().to_str().unwrap().to_string())
        );
        assert_eq!(rss.last_updated, _RSS_LAST_UPDATED);
    }

    #[tokio::test]
    async fn test_sync_channel() {
        let (mut _m, mut rss) = _build_from_mock().await;
        let _m = _m
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/rss+xml")
            .with_body(_RSS_UPDATED)
            .create();

        rss.sync().await.unwrap();

        assert_eq!(rss.last_updated, _RSS_UPDATED_LAST_UPDATED);
    }

    pub async fn _build_from_mock() -> (mockito::ServerGuard, Rss) {
        let mut _m = mockito::Server::new();
        _m.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/rss+xml")
            .with_body(_RSS)
            .create();
        let addr = _m.host_with_port();

        let rss = Rss::from_url(format!("http://{}", addr).as_str())
            .await
            .unwrap();

        (_m, rss)
    }

    pub fn _build_from_tempfile() -> (NamedTempFile, Rss) {
        let file = tempfile::NamedTempFile::new().unwrap();
        file.as_file().write_all(_RSS.as_bytes()).unwrap();
        let rss =
            Rss::from_file(file.path().to_str().unwrap()).expect("enable to open file as rss");

        (file, rss)
    }

    pub const _RSS_LAST_UPDATED: &'static str = "Mon, 06 Sep 2010 00:01:00 +0000";
    pub const _RSS: &'static str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
            <channel>
                <title>Sample Feed</title>
                <link>http://example.com/</link>
                <description>Sample feed</description>
                <lastBuildDate>Mon, 06 Sep 2010 00:01:00 +0000</lastBuildDate>
                <item>
                    <title>Item 1</title>
                    <link>http://example.com/item1</link>
                    <description>Item 1 description</description>
                    <pubDate>Mon, 06 Sep 2010 16:20:00 +0000</pubDate>
                </item>
                <item>
                    <title>Item 2</title>
                    <link>http://example.com/item2</link>
                    <description>Item 2 description</description>
                    <pubDate>Mon, 06 Sep 2010 16:20:00 +0000</pubDate>
                </item>
            </channel>
        </rss>
        "#;
    pub const _RSS_UPDATED_LAST_UPDATED: &'static str = "Mon, 07 Sep 2010 00:20:00 +0000";
    pub const _RSS_UPDATED: &'static str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
            <channel>
                <title>Sample Feed</title>
                <link>http://example.com/</link>
                <description>Sample feed</description>
                <lastBuildDate>Mon, 07 Sep 2010 00:20:00 +0000</lastBuildDate>
                <item>
                    <title>Item 1</title>
                    <link>http://example.com/item1</link>
                    <description>Item 1 description</description>
                    <pubDate>Mon, 06 Sep 2010 16:20:00 +0000</pubDate>
                </item>
                <item>
                    <title>Item 2</title>
                    <link>http://example.com/item2</link>
                    <description>Item 2 description</description>
                    <pubDate>Mon, 06 Sep 2010 16:20:00 +0000</pubDate>
                </item>
                <item>
                    <title>Item 3</title>
                    <link>http://example.com/item3</link>
                    <description>Item 3 description</description>
                    <pubDate>Mon, 07 Sep 2010 16:20:00 +0000</pubDate>
                </item>
            </channel>
        </rss>
        "#;
}
