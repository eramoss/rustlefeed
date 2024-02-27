use std::str::FromStr;

use feed_rs::model::Entry;
use rusqlite::Row;

pub trait Parser {
    fn into_html(&self) -> String;
    fn from_row(row: Row) -> Self;
}

impl Parser for Entry {
    fn into_html(&self) -> String {
        let mut html = String::new();
        html.push_str(&format!(
            "<h1 class=\"feed-item-title\">{}</h1>",
            self.title.clone().unwrap_or_default().content
        ));
        html.push_str(&format!("<p>{}</p>", self.published.unwrap_or_default()));
        html.push_str(&format!(
            "<p class=\"feed-item-summary\">{}</p>",
            self.authors
                .iter()
                .map(|author| author.name.clone())
                .collect::<Vec<String>>()
                .join(", ")
        ));
        html.push_str(&format!(
            "<p class=\"feed-item-summary\">{}</p>",
            self.categories
                .iter()
                .map(|c| c.term.clone())
                .collect::<Vec<String>>()
                .join(", ")
        ));

        html.push_str(&format!(
            "<p class=\"feed-item-summary\">{}</p>",
            self.summary.clone().unwrap_or_default().content
        ));

        html.push_str(&format!(
            "<p class=\"feed-item-content\">{}</p>",
            self.content
                .clone()
                .unwrap_or_default()
                .body
                .unwrap_or_default()
        ));
        if let Some(link) = self.links.get(0) {
            html.push_str(&format!(
                "<a  class=\"feed-item-link\" href='{}'>Read more</a>",
                link.href
            ));
        }
        html
    }
    fn from_row(row: Row) -> Self {
        Self {
            id: row.get("id").unwrap_or_default(),
            title: Some(feed_rs::model::Text {
                content: row.get("title").unwrap_or_default(),
                content_type: mime::Mime::from_str("*/*").unwrap(),
                src: None,
            }),
            summary: Some(feed_rs::model::Text {
                content: row.get("summary").unwrap_or_default(),
                content_type: mime::Mime::from_str("*/*").unwrap(),
                src: None,
            }),
            content: Some(feed_rs::model::Content {
                body: Some(row.get("content").unwrap_or_default()),
                content_type: mime::Mime::from_str("*/*").unwrap(),
                length: None,
                src: None,
            }),
            authors: vec![feed_rs::model::Person {
                email: None,
                name: row.get("author").unwrap_or_default(),
                uri: None,
            }],
            categories: vec![feed_rs::model::Category {
                label: None,
                scheme: None,
                term: row.get("categories").unwrap_or_default(),
            }],
            links: vec![feed_rs::model::Link {
                href: row.get("link").unwrap_or_default(),
                length: None,
                rel: None,
                title: None,
                media_type: None,
                href_lang: None,
            }],
            language: Some(row.get("language").unwrap_or_default()),
            ..Default::default()
        }
    }
}
