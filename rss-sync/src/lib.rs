use std::{fs::File, io::BufReader};

use rss::Channel;

struct RssSync {
    origin: Origin,
    channel: Channel,
    last_updated: String,
}

#[derive(PartialEq, Debug)]
enum Origin {
    Url(String),
    File(String),
}

impl RssSync {
    pub async fn from_url(url: &str) -> Result<RssSync, Box<dyn std::error::Error>> {
        let content = reqwest::get(url).await?.bytes().await?;

        let channel = Channel::read_from(&content[..])?;
        let last_updated = channel.last_build_date().unwrap().to_string();
        Ok(RssSync {
            origin: Origin::Url(url.to_string()),
            channel,
            last_updated,
        })
    }

    pub fn from_file(filepath: &str) -> Result<RssSync, Box<dyn std::error::Error>> {
        let file = File::open(filepath).unwrap();
        let channel = Channel::read_from(BufReader::new(file)).unwrap();
        let last_updated = channel.last_build_date().unwrap().to_string();
        Ok(RssSync {
            origin: Origin::File(filepath.to_string()),
            channel,
            last_updated,
        })
    }

    pub async fn sync_channel(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.origin {
            Origin::Url(url) => {
                let content = reqwest::get(url).await?.bytes().await?;
                let channel = Channel::read_from(&content[..])?;
                let last_updated = channel.last_build_date().unwrap().to_string();
                if last_updated != self.last_updated {
                    println!("Channel updated");
                }
            }
            Origin::File(filepath) => {
                let file = File::open(filepath).unwrap();
                let channel = Channel::read_from(BufReader::new(file)).unwrap();
                let last_updated = channel.last_build_date().unwrap().to_string();
                if last_updated != self.last_updated {
                    println!("Channel updated");
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_rss_sync_from_url() {
        let mut _m = mockito::Server::new();
        _m.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/rss+xml")
            .with_body(_RSS)
            .create();
        let addr = _m.host_with_port();
        let rss_sync = RssSync::from_url(format!("http://{}", addr).as_str())
            .await
            .unwrap();
        assert_eq!(rss_sync.origin, Origin::Url(format!("http://{}", addr)));
        assert_eq!(rss_sync.last_updated, _RSS_LAST_UPDATED);
    }

    #[test]
    fn test_rss_sync_from_file() {
        let file = tempfile::NamedTempFile::new().unwrap();
        file.as_file().write_all(_RSS.as_bytes()).unwrap();
        let rss_sync =
            RssSync::from_file(file.path().to_str().unwrap()).expect("enable to open file as rss");
        assert_eq!(
            rss_sync.origin,
            Origin::File(file.path().to_str().unwrap().to_string())
        );
        assert_eq!(rss_sync.last_updated, _RSS_LAST_UPDATED);
    }

    const _RSS_LAST_UPDATED: &'static str = "Mon, 06 Sep 2010 00:01:00 +0000";
    const _RSS: &'static str = r#"
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
}
