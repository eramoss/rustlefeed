#[macro_use]
extern crate rocket;
use rocket::State;
use rocket::{form::Form, response::content::RawHtml};
use rss_sync::{parser::Parser, rss_manager::RssManager, Rss};
use std::sync::{Arc, Mutex};

#[get("/")]
fn index() -> &'static str {
    "Click the button to view HTML pages!"
}

#[get("/next")]
fn next_page(state: &StateApp) -> Option<RawHtml<String>> {
    let mg = state.lock().unwrap();
    let current_item = mg.all_news.last();
    match current_item {
        None => return None,
        Some(item) => Some(RawHtml(format!(
            "{}{}{}",
            item.clone().into_html().unwrap(),
            "            
            <form action=\"\" method=\"post\">
            <input type=\"text\" name=\"like\" value=true>
                    <button type=\"submit\">Like</button>
                  </form>
                  <form action=\"\" method=\"post\">
                  <input type=\"text\" name=\"like\" value=false>
                    <button type=\"submit\">Unlike</button>
                  </form>",
            STYLE,
        ))),
    }
}

#[derive(FromForm)]
struct FormData {
    like: bool,
}

#[post("/next", data = "<is_liked>")]
fn load_next_page(is_liked: Form<FormData>, state: &StateApp) -> Option<RawHtml<String>> {
    let mut mg = state.lock().unwrap();
    let previous_item = mg.all_news.pop();
    if let Some(item) = previous_item {
        if is_liked.like {
            mg.already_seen.insert(1, item.clone());
        } else {
            mg.already_seen.insert(0, item.clone());
        }
    }

    let current_item = mg.all_news.last();
    match current_item {
        None => return None,
        Some(item) => Some(RawHtml(format!(
            "{}{}{}",
            item.clone().into_html().unwrap(),
            "<form action=\"\" method=\"post\">
            <input type=\"text\" name=\"like\" value=true>
                    <button type=\"submit\">Like</button>
                  </form>
                  <form action=\"\" method=\"post\">
                  <input type=\"text\" name=\"like\" value=false>
                    <button type=\"submit\">Unlike</button>
                  </form>",
            STYLE,
        ))),
    }
}

type StateApp = State<Arc<Mutex<RssManager>>>;
#[launch]
async fn rocket() -> _ {
    let manager = build_manager().await;
    let state = Arc::new(Mutex::new(manager));
    let closer = Arc::clone(&state);

    rocket::build()
        .manage(state)
        .mount("/", routes![index, next_page, load_next_page])
        .attach(rocket::fairing::AdHoc::on_shutdown(
            "save db on shutdown",
            |_rocket| {
                Box::pin(async move {
                    closer
                        .lock()
                        .unwrap()
                        .save_to_database("db/RssHist.db")
                        .unwrap();
                })
            },
        ))
}

async fn build_manager() -> RssManager {
    let rss1 = Rss::from_url("https://mashable.com/feeds/rss/all")
        .await
        .unwrap();
    let rss2 = Rss::from_url("https://podcastfeeds.nbcnews.com/RPWEjhKq")
        .await
        .unwrap();

    let mut manager = RssManager::new();
    manager.add_feed("Mashable".to_string(), rss1);
    manager.add_feed("NBC News".to_string(), rss2);
    manager.sync_all().await.unwrap();
    manager.update_all_news();
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
