use rss::Item;

pub trait Parser {
    fn into_html(self) -> Result<String, Box<dyn std::error::Error>>;
    fn into_json(self) -> Result<String, Box<dyn std::error::Error>>;
}

impl Parser for Item {
    fn into_html(self) -> Result<String, Box<dyn std::error::Error>> {
        let title = self.title().unwrap();
        let link = self.link().unwrap();
        let description = self.description().unwrap();
        Ok(format!(
            "<h1>{}</h1><a href=\"{}\">{}</a><p>{}</p>
            ",
            title, link, link, description
        ))
    }
    fn into_json(self) -> Result<String, Box<dyn std::error::Error>> {
        todo!()
    }
}
