#[macro_use]
extern crate rocket;
use feed_sync::{parser::Parser, FeedManager};
use naive_classifier::NaiveBayesClassifier;

use rocket::http::Status;
use rocket::response::status::Custom;
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
    let mut manager = state.manager.lock().unwrap();

    if let Some(is_liked) = msg.liked {
        let last = manager.to_see.pop().unwrap().clone();
        manager.already_seen.push((last, is_liked));
    }

    loop {
        let current = manager.to_see.last();
        if current.is_none() {
            return RawHtml("No more entries".to_string());
        }
        let current = current.unwrap();
        let possibility_of_like = state.classifier.lock().unwrap().classify(current.clone());
        if possibility_of_like >= 0.5 {
            return RawHtml(current.into_html());
        } else {
            manager.to_see.pop();
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AddFeedReq {
    url: String,
}

#[post("/add-feed", data = "<feed_url>")]
async fn add_feed(state: &StateApp, feed_url: Json<AddFeedReq>) -> Custom<Json<String>> {
    let mut manager = state.manager.lock().unwrap().clone();
    if manager.get_feed(&feed_url.url).is_some() {
        return Custom(Status::BadRequest, Json("Feed already added".to_string()));
    }
    let result = manager.new_feed(&feed_url.url).await;

    *state.manager.lock().unwrap() = manager.clone();

    if result.is_err() {
        return Custom(Status::BadRequest, Json("Error adding feed".to_string()));
    }
    Custom(
        Status::Accepted,
        Json("Feed addition task started".to_string()),
    )
}

#[derive(Serialize, Deserialize)]
struct FeedJson {
    title: String,
    url: String,
}

#[post("/delete-feed", data = "<feed_url>")]
async fn delete_feed(state: &StateApp, feed_url: Json<AddFeedReq>) -> Custom<Json<String>> {
    let mut manager = state.manager.lock().unwrap().clone();
    manager.remove_feed_by_url(&feed_url.url);
    *state.manager.lock().unwrap() = manager.clone();
    assert!(state.manager.lock().unwrap().feeds.len() == manager.feeds.len());
    Custom(
        Status::Accepted,
        Json("Feed deletion task started".to_string()),
    )
}

#[get("/feeds")]
async fn list_feeds(state: &StateApp) -> Json<Vec<FeedJson>> {
    let mut feeds = vec![];
    for (feed, url) in state.manager.lock().unwrap().feeds.iter() {
        let f = FeedJson {
            title: <std::option::Option<feed_rs::model::Text> as Clone>::clone(&feed.title)
                .unwrap_or_default()
                .content
                .clone(),
            url: url.clone(),
        };
        feeds.push(f);
    }
    Json(feeds)
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

struct StateAppS {
    manager: Arc<Mutex<FeedManager>>,
    classifier: Arc<Mutex<NaiveBayesClassifier>>,
}
type StateApp = State<StateAppS>;
#[launch]
async fn rocket() -> _ {
    let manager = Arc::new(Mutex::new(build_manager().await));
    let state = StateAppS {
        manager: Arc::clone(&manager),
        classifier: Arc::new(Mutex::new(
            NaiveBayesClassifier::new("db/FeedHistory.db").unwrap(),
        )),
    };
    let closer = Arc::clone(&manager);

    rocket::build()
        .manage(state)
        .mount(
            "/",
            routes![index, next, add_feed, files, list_feeds, delete_feed],
        )
        .attach(fairing::AdHoc::on_shutdown(
            "saving already seen on db",
            |_rocket| {
                Box::pin(async move {
                    closer
                        .lock()
                        .unwrap()
                        .save_already_seen("db/FeedHistory.db")
                        .unwrap();

                    closer
                        .lock()
                        .unwrap()
                        .save_feeds("db/FeedHistory.db")
                        .unwrap();
                })
            },
        ))
}

async fn build_manager() -> FeedManager {
    let mut manager = FeedManager::new();
    manager.load_feeds_from_db("db/FeedHistory.db").unwrap();
    manager.sync().await;
    manager
}
