#[macro_use]
extern crate rocket;
use feed_sync::{parser::Parser, FeedManager};
use rocket::{response::content::RawHtml, State};
use std::sync::{Arc, Mutex};

#[get("/")]
fn index(state: &StateApp) -> RawHtml<String> {
    let manager = state.lock().unwrap();
    let first_entry = manager.to_see.first().unwrap();
    let mut html = first_entry.into_html();
    html.push_str(STYLE);
    RawHtml(html)
}

type StateApp = State<Arc<Mutex<FeedManager>>>;
#[launch]
async fn rocket() -> _ {
    let manager = build_manager().await;
    let state = Arc::new(Mutex::new(manager));

    rocket::build()
        .manage(state)
        .mount("/", routes![index])
        .attach(rocket::fairing::AdHoc::on_shutdown(
            "save db on shutdown",
            |_rocket| {
                Box::pin(async move {
                    // closer
                    //     .lock()
                    //     .unwrap()
                    //     .save_to_database("db/RssHist.db")
                    //     .unwrap();
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

const STYLE: &'static str = r#"
<style>
* { margin:0; padding:0; }

html { display:block; background-color:#960000; padding-bottom:50px; }
body { font:80% Verdana, sans-serif; color:#000; background:#fff url(pagebg_rss.jpg) top left no-repeat; padding:25px 0 0 35px; }

a { color:#960000; }
a:hover { text-decoration:none; }

h2 { font-weight:normal; border-bottom:1px solid #960000; margin-bottom:0.4em; }
h2 a { display:block; margin-bottom:0.2em; text-decoration:none; color:#000; }

div { line-height:1.6em; }

div#content { background:#fff url(logo.jpg) bottom right no-repeat; margin-right:15px; padding:1em 0 55px 0; }
div#content div { margin:0 1em 1em 0; }
img {
    max-width: 30%;
    height: auto;
}
div#explanation { padding:1em 1em 0 1em; border:1px solid #ddd; background:#efefef; margin-right:2em; }
div#explanation h1 { font-weight:normal; font-size:1.8em; margin-bottom:0.3em; }
div#explanation p { margin-bottom:1em; }

button {
    margin-top: 10px;
    padding: 10px 20px;
}
input {
    display: none;
}
<style/>
"#;
