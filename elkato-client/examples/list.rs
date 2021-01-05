use chrono::{Duration, Local, Utc};
use elkato_client::{Client, ListOptions};
use elkato_client::{Config, User};
use elkato_common::data::Booking;
use futures::{StreamExt, TryStreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = Client::new(Config {
        url: "https://www.elkato.de".parse()?,
    })?;

    let now = Local::now().with_timezone(&Utc);

    let bookings: Vec<Booking> = client
        .list_bookings(
            User {
                club: "demo".into(),
                username: "demo".into(),
                password: Some("demo".into()),
            },
            ListOptions {
                owner: Some("demo".into()),
                start_from: Some(now.date() - Duration::days(7)),
                end_to: Some(now.date() + Duration::days(7)),
                ..Default::default()
            },
        )
        .boxed()
        .try_collect()
        .await?;

    for booking in &bookings {
        println!("{:?}", booking);
    }

    println!("Found {} bookings", bookings.len());

    Ok(())
}
