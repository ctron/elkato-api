mod data;
mod parser;

use data::Booking;

fn parse(body: String) -> Result<Vec<Booking>, ()> {
    parser::parse_query(body)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let resp = client
        .get("https://www.elkato.de/buchung/search.php")
        .basic_auth("demo", Some("demo"))
        .header("Accept-Language", "de-DE;de;q=0.5")
        .query(&[
            ("club", "demo"),
            ("sel_room", "all"),
            ("sel_booker", "all"),
            ("sel_owner", "all"),
            ("active", "on"),
        ])
        .send()
        .await?;

    println!("URL: {:?}", resp.url());
    println!("{:#?}", resp);

    let result = parse(resp.text().await?);
    println!("{:#?}", result);

    Ok(())
}
