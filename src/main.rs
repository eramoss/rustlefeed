#[macro_use]
extern crate rocket;
use feed_sync::{parser::Parser, FeedManager};
use rocket::{
    fairing,
    fs::NamedFile,
    response::content::RawHtml,
    serde::{json::Json, Deserialize, Serialize},
    State,
};
use std::{
    path::{Path, PathBuf},
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

#[get("/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("assets").join(file)).await.ok()
}

#[get("/")]
async fn index(_manager: &StateApp) -> Option<NamedFile> {
    let file = NamedFile::open("assets/index.html").await.unwrap();
    Some(file)
}

type StateApp = State<Arc<Mutex<FeedManager>>>;
#[launch]
async fn rocket() -> _ {
    let manager = build_manager().await;
    let state = Arc::new(Mutex::new(manager));
    let closer = Arc::clone(&state);

    rocket::build()
        .manage(state)
        .mount("/", routes![index, next, files])
        .attach(fairing::AdHoc::on_shutdown(
            "saving already seen on db",
            |_rocket| {
                Box::pin(async move {
                    closer
                        .lock()
                        .unwrap()
                        .save_already_seen("db/FeedHistory.db")
                        .unwrap();
                })
            },
        ))
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
