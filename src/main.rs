#[macro_use]
extern crate rocket;
use feed_sync::{parser::Parser, FeedManager};
use rocket::{
    response::content::RawHtml,
    serde::{json::Json, Deserialize, Serialize},
    State,
};
use std::{
    fs::File,
    sync::{Arc, Mutex},
};

#[derive(Serialize, Deserialize)]
struct IsLiked {
    liked: Option<bool>,
}

#[post("/next", data = "<msg>")]
fn next(state: &StateApp, msg: Json<IsLiked>) -> RawHtml<String> {
    let mut manager = state.lock().unwrap();

    if let Some(is_liked) = msg.liked {
        let last = manager.to_see.pop().unwrap().clone();
        manager.already_seen.push((last, is_liked));
    }

    let current = manager.to_see.last().unwrap().clone();
    let html = current.into_html();
    RawHtml(html)
}

#[get("/")]
fn index(_manager: &StateApp) -> RawHtml<File> {
    let file = File::open("index.html").unwrap();
    RawHtml(file)
}

type StateApp = State<Arc<Mutex<FeedManager>>>;
#[launch]
async fn rocket() -> _ {
    let manager = build_manager().await;
    let state = Arc::new(Mutex::new(manager));

    rocket::build()
        .manage(state)
        .mount("/", routes![index, next])
}

async fn build_manager() -> FeedManager {
    let mut manager = FeedManager::new();
    manager
        .new_feed("https://mashable.com/feeds/rss/all")
        .await
        .unwrap();
    manager
        .new_feed("https://podcastfeeds.nbcnews.com/RPWEjhKq")
        .await
        .unwrap();

    manager
}
