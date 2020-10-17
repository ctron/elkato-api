use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};

use elkato_client::{Client, Config, ListOptions, User};
use serde_json::json;

use actix_cors::Cors;
use actix_web_httpauth::extractors::basic;
use actix_web_httpauth::extractors::basic::BasicAuth;
use chrono::{Duration, Local, Utc};
use futures::stream::TryStreamExt;
use futures::StreamExt;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/{club}/bookings/current")]
async fn list_current_bookings(
    club: web::Path<String>,
    client: web::Data<Client>,
    auth: BasicAuth,
) -> Result<impl Responder, actix_web::Error> {
    let user = User {
        club: club.0,
        username: auth.user_id().to_string(),
        password: auth.password().map(|s| s.to_string()),
    };

    log::info!("Get current");

    let now = Local::now().with_timezone(&Utc);

    log::info!("Now: {}", now);

    let result: Vec<_> = client
        .list_bookings(
            user,
            ListOptions {
                owner: Some(auth.user_id().to_string()),
                start_from: Some(now.date() - Duration::days(7)),
                end_to: Some(now.date() + Duration::days(7)),
                ..Default::default()
            },
        )
        .boxed()
        .try_collect()
        .await
        .map_err(|e| HttpResponse::InternalServerError().json(json!({"message": e.to_string()})))?;

    Ok(HttpResponse::Ok().json(result))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let client = elkato_client::Client::new(Config {
        url: "https://www.elkato.de".parse()?,
    })?;

    let addr = std::env::var("BIND_ADDR").ok();
    let addr = addr.as_ref().map(|s| s.as_str());

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(Cors::new().send_wildcard().finish())
            .data(basic::Config::default().realm("Elkato Proxy"))
            .data(web::JsonConfig::default().limit(4096))
            .data(client.clone())
            .service(index)
            .service(health)
            .service(list_current_bookings)
    })
    .bind(addr.unwrap_or("127.0.0.1:8080"))?
    .run()
    .await?;

    Ok(())
}
