#[macro_use]
/*
    Expands macro visibility, or imports macros from other crates.
*/
extern crate rocket;
/*
    extern crate foo - indicates that you want to link against an external library and brings the top-level crate name into scope (equivalent to use foo).
    As of Rust 2018, in most cases you won't need to use extern crate anymore because Cargo informs the compiler about what crates are present.
    The difference between the two is subtle: The entry in the cargo.toml declares a dependency on a package, while extern c
*/

use rocket::form::Form; /* A struct that provides a generic means to parse arbitrary structures from incoming form data. */
use rocket::fs::{relative, FileServer};
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::{Shutdown, State};

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
struct Messsage {
    #[field(validate = len(..30))]
    pub room: String,
    #[field(validate = len(..30))]
    pub username: String,
    pub message: String,
}

#[get("/events")]
async fn events(queue: &State<Sender<Messsage>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };
            yield Event::json(&msg);
        }
    }
}

#[post("/message", data = "<form>")]
async fn post(form: Form<Messsage>, queue: &State<Sender<Messsage>>) {
    let _res = queue.send(form.into_inner());
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(channel::<Messsage>(1024).0)
        .mount("/", routes![post, events])
        .mount("/", FileServer::from(relative!("static")))
}
