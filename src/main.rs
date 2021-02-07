mod auth;
mod models;
mod schema;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;

use crate::auth::BasicAuth;
mod repositories;

use diesel::prelude::*;
use models::*;
use repositories::RustaceanRepository;
use schema::*;

use rocket::{http::ext::IntoCollection, response::status};
use rocket_contrib::json::Json;
use rocket_contrib::json::JsonValue;

#[database("sql_path")]
struct DbConn(diesel::SqliteConnection);

#[get("/rustaceans")]
async fn get_rustaceans(_auth: BasicAuth, conn: DbConn) -> JsonValue {
    conn.run(|c| {
        let all = RustaceanRepository::load_all(c).expect("Error loading from database!");
        json!(all)
    })
    .await
}
#[get("/rustaceans/<id>")]
async fn view_rustaceans(id: i32, _auth: BasicAuth, conn: DbConn) -> JsonValue {
    conn.run(move |c| {
        let rustacean = RustaceanRepository::find(c, id).expect("Error loading from database!");
        json!(rustacean)
    })
    .await
}
#[post("/rustaceans", format = "json", data = "<new_rustacean>")]
async fn create_rustaceans(
    _auth: BasicAuth,
    conn: DbConn,
    new_rustacean: Json<NewRustacean>,
) -> JsonValue {
    conn.run(|c| {
        let result =
            RustaceanRepository::create(c, new_rustacean.into_inner()).expect("Error inserting");
        json!(result)
    })
    .await
}
#[put("/rustaceans/<_id>", format = "json", data = "<rustacean>")]
async fn update_rustaceans(
    _id: i32,
    _auth: BasicAuth,
    conn: DbConn,
    rustacean: Json<Rustacean>,
) -> JsonValue {
    conn.run(move |c| {
        let result = RustaceanRepository::update(c, rustacean.into_inner())
            .expect("Error updating");
        json!(result)
    })
    .await
}
#[delete("/rustaceans/<id>")]
async fn delete_rustaceans(id: i32, _auth: BasicAuth, conn: DbConn) -> status::NoContent {
    conn.run(move |c| {
        RustaceanRepository::delete(c, id)
            .expect("Error deleting");
        status::NoContent
    })
    .await
}

#[catch(401)]
fn not_autherized() -> JsonValue {
    json!("Not Autherized!!")
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!("Not found!!")
}

#[rocket::main]
async fn main() {
    let _ = rocket::ignite()
        .mount(
            "/",
            routes![
                get_rustaceans,
                view_rustaceans,
                create_rustaceans,
                update_rustaceans,
                delete_rustaceans
            ],
        )
        .register(catchers![not_found, not_autherized])
        .attach(DbConn::fairing())
        .launch()
        .await;
}
