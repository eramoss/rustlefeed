#[macro_use]
extern crate rocket;

use rocket::response::content::RawHtml;
use rocket::State;
use rss_sync::{parser::Parser, rss_manager::RssManager, Rss};
use std::sync::{Arc, Mutex};

// Define a struct to hold the HTML pages
#[derive(Debug)]
struct HtmlPages {
    pages: Arc<Mutex<Vec<String>>>,
}

#[get("/")]
fn index() -> &'static str {
    "Click the button to view HTML pages!"
}

#[get("/next")]
fn next_page(pages: &State<HtmlPages>) -> Option<RawHtml<String>> {
    // Lock the mutex to access the pages vector
    let pages = pages.pages.lock().unwrap();

    // Retrieve the next page if available
    let current_page = pages.first()?;

    // Clone the current page content to serve
    let page_content = current_page.clone();

    Some(RawHtml(page_content))
}

#[post("/next")]
fn load_next_page(pages: &State<HtmlPages>) -> RawHtml<String> {
    // Lock the mutex to access the pages vector
    let mut pages = pages.pages.lock().unwrap();

    // Remove the current page from the vector
    if !pages.is_empty() {
        pages.remove(0);
    }
    let warning = &String::from("No more pages to load!");
    if let Some(page) = pages.first() {
        let mut page = page.clone();
        page.push_str(
            "
        <form id=\"postForm\" method=\"post\" action=\"\">
       
        <button type=\"submit\">Next</button>
        </form>",
        );
        return RawHtml(page);
    } else {
        return RawHtml(warning.clone());
    }
}

#[launch]
async fn rocket() -> _ {
    let rss1 = Rss::from_url("https://mashable.com/feeds/rss/all")
        .await
        .unwrap();
    let rss2 = Rss::from_url("https://podcastfeeds.nbcnews.com/RPWEjhKq")
        .await
        .unwrap();
    dbg!(&rss2.news);
    let mut manager = RssManager::new();
    manager.add_feed("Mashable".to_string(), rss1);
    manager.add_feed("NBC News".to_string(), rss2);
    manager.sync_all().await.unwrap();
    manager.update_all_news();

    dbg!(&manager.all_news);

    let news = manager.all_news;
    let new_as_html: Vec<String> = news
        .iter()
        .map(|item| item.clone().into_html().unwrap())
        .collect();

    let htmls = HtmlPages {
        pages: Arc::new(Mutex::new(new_as_html)),
    };
    rocket::build()
        .manage(htmls)
        .mount("/", routes![index, next_page, load_next_page])
}
