#[macro_use]
extern crate rocket;
use feed_sync::{parser::Parser, FeedManager};
use naive_classifier::NaiveBayesClassifier;
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
        let possible_likes = state.classifier.lock().unwrap().classify(current.clone());
        if possible_likes {
            return RawHtml(current.into_html());
        } else {
            manager.to_see.pop();
        }
    }
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
