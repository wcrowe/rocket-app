mod auth;
mod models;
mod schema;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use crate::auth::BasicAuth;
mod repositories;

use models::*;
use repositories::RustaceanRepository;

use rocket::{fairing::AdHoc, http::Status, response::status, Rocket};

use rocket_contrib::json::Json;
use rocket_contrib::json::JsonValue;

embed_migrations!();

#[database("sql_path")]
struct DbConn(diesel::SqliteConnection);

#[get("/rustaceans")]
async fn get_rustaceans(
    _auth: BasicAuth,
    conn: DbConn,
) -> Result<JsonValue, status::Custom<JsonValue>> {
    conn.run(|c| {
        RustaceanRepository::load_all(c)
            .map(|rustacean| json!(rustacean))
            .map_err(|_| {
                status::Custom(
                    Status::InternalServerError,
                    json!("Server Error".to_string()),
                )
            })
    })
    .await
}
#[get("/rustaceans/<id>")]
async fn view_rustaceans(
    id: i32,
    _auth: BasicAuth,
    conn: DbConn,
) -> Result<JsonValue, status::Custom<JsonValue>> {
    conn.run(move |c| {
        RustaceanRepository::find(c, id)
            .map(|rustacean| json!(rustacean))
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    })
    .await
}
#[post("/rustaceans", format = "json", data = "<new_rustacean>")]
async fn create_rustaceans(
    _auth: BasicAuth,
    conn: DbConn,
    new_rustacean: Json<NewRustacean>,
) -> Result<JsonValue, status::Custom<JsonValue>> {
    conn.run(|c| {
        RustaceanRepository::create(c, new_rustacean.into_inner())
            .map(|rustacean| json!(rustacean))
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    })
    .await
}
#[put("/rustaceans/<_id>", format = "json", data = "<rustacean>")]
async fn update_rustaceans(
    _id: i32,
    _auth: BasicAuth,
    conn: DbConn,
    rustacean: Json<Rustacean>,
) -> Result<JsonValue, status::Custom<JsonValue>> {
    conn.run(move |c| {
        RustaceanRepository::update(c, rustacean.into_inner())
            .map(|rustacean| json!(rustacean))
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    })
    .await
}
#[delete("/rustaceans/<id>")]
async fn delete_rustaceans(
    id: i32,
    _auth: BasicAuth,
    conn: DbConn,
) -> Result<status::NoContent, status::Custom<JsonValue>> {
    conn.run(move |c| {
        RustaceanRepository::delete(c, id)
            .map(|_| status::NoContent)
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
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

#[catch(500)]
fn internal_error() -> &'static str {
    "Whoops! Looks like we messed up."
}

async fn run_db_migrations(rocket: Rocket) -> Result<Rocket, Rocket> {
    DbConn::get_one(&rocket).await
        .expect("database connection")
        .run(|c| match embedded_migrations::run(c) {
            Ok(()) => Ok(rocket),
            Err(e) => {
                error!("Failed to run database migrations: {:?}", e);
                Err(rocket)
            }
        }).await
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
        .register(catchers![not_found, not_autherized, internal_error])
        .attach(DbConn::fairing())
        .attach(AdHoc::on_attach("Database Migrations", run_db_migrations))
        .launch()
        .await;
}
