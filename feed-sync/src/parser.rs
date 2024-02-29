use feed_rs::model::Entry;

pub trait Parser {
    fn into_html(&self) -> String;
}

impl Parser for Entry {
    fn into_html(&self) -> String {
        let mut html = String::new();
        let summary = self.summary.clone().unwrap_or_default().content;
        let mut content = self
            .content
            .clone()
            .unwrap_or_default()
            .body
            .unwrap_or_default();
        if content == summary {
            content = "".to_string();
        }
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

        html.push_str(&format!("<p class=\"feed-item-content\">{}</p>", content));
        if let Some(link) = self.links.get(0) {
            html.push_str(&format!(
                "<a  class=\"feed-item-link\" href='{}'>Read more</a>",
                link.href
            ));
        }
        html
    }
}
