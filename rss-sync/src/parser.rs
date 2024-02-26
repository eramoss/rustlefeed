use rss::Item;

pub trait Parser {
    fn into_html(self) -> Result<String, Box<dyn std::error::Error>>;
    fn into_json(self) -> Result<String, Box<dyn std::error::Error>>;
}

impl Parser for Item {
    fn into_html(self) -> Result<String, Box<dyn std::error::Error>> {
        let author = self.author().unwrap_or("Unknown");
        let categories = self
            .categories()
            .to_vec()
            .iter()
            .fold(String::new(), |acc, x| acc + &x.name() + ", ");
        let pub_date = self.pub_date().unwrap_or("Unknown");
        let mut content = self.content().unwrap_or_default();
        let title = self.title().unwrap_or_default();
        let link = self.link().unwrap_or_default();
        let description = self.description().unwrap_or_default();
        if content == description {
            content = "";
        }

        let html = format!(
            "
        <h1>{}</h1>
        <p><strong>Author:</strong> {}</p>
        <p><strong>Categories:</strong> {}</p>
        <p><strong>Publication Date:</strong> {}</p>
        <p><strong>Description:</strong> {}</p>
        <p>{}</p>
        <p><a href=\"{}\">Read more</a></p> 
    ",
            title, author, categories, pub_date, description, content, link
        );
        Ok(html)
    }
    fn into_json(self) -> Result<String, Box<dyn std::error::Error>> {
        todo!()
    }
}
