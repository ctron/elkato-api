use crate::parser;
use anyhow::Context;
use chrono::{Date, Datelike, Utc};
use elkato_common::data::Booking;
use futures::TryStream;
use futures::{stream, TryStreamExt};
use serde::{Deserialize, Serialize};
use url::{ParseError, Url};

#[cfg(feature = "reqwest")]
use reqwest::header::{self, HeaderValue};
#[cfg(feature = "reqwest")]
use tokio::stream::StreamExt;

#[cfg(feature = "reqwest")]
#[derive(Clone, Debug)]
pub struct Client {
    config: Config,
    client: reqwest::Client,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub url: Url,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub club: String,
    pub username: String,
    pub password: Option<String>,
}

#[derive(Clone, Debug)]
pub enum BookingState {
    Active,
    Inactive,
    All,
}

impl Default for BookingState {
    fn default() -> Self {
        BookingState::Active
    }
}

#[derive(Clone, Debug, Default)]
pub struct ListOptions {
    pub owner: Option<String>,
    pub start_from: Option<Date<Utc>>,
    pub start_to: Option<Date<Utc>>,
    pub end_from: Option<Date<Utc>>,
    pub end_to: Option<Date<Utc>>,
    pub state: BookingState,
}

fn date_filter_to_query(prefix: &str, date: Option<Date<Utc>>) -> Vec<(String, String)> {
    match date {
        Some(d) => vec![
            (prefix.to_string(), "1".into()),
            (format!("{}_day", prefix), d.day().to_string()),
            (format!("{}_month", prefix), d.month().to_string()),
            (format!("{}_year", prefix), d.year().to_string()),
        ],
        None => vec![(prefix.to_string(), "0".into())],
    }
}

#[cfg(feature = "reqwest")]
impl Client {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let mut headers = header::HeaderMap::new();

        headers.insert(
            "Accept-Language",
            HeaderValue::from_static("de-DE;de;q=0.5"),
        );

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Client { config, client })
    }

    pub fn list_bookings(
        &self,
        user: User,
        options: ListOptions,
    ) -> impl TryStream<Item = Result<Booking, anyhow::Error>> + '_ {
        #[derive(Clone)]
        struct ListState {
            offset: Option<usize>,
            client: reqwest::Client,
            user: User,
            url: Result<Url, ParseError>,
            options: ListOptions,
        }

        let url = self.config.url.join("/buchung/search.php");
        let client = self.client.clone();

        let init = ListState {
            offset: Some(0),
            url,
            client,
            user,
            options,
        };

        stream::try_unfold(init, move |state| {
            async move {
                let next = state.clone();

                match state.offset {
                    // having no offset means, we finish up in the last iteration
                    None => Result::<_, anyhow::Error>::Ok(None),
                    // having an offset means we need to pull in more data
                    Some(offset) => {
                        let builder = state
                            .client
                            .get(state.url.clone()?)
                            .basic_auth(state.user.username.clone(), state.user.password.clone())
                            .query(&[
                                ("club", state.user.club.clone()),
                                ("search_pos", format!("{}", offset)),
                                ("sel_room", "all".into()),
                                ("sel_booker", "all".into()),
                                (
                                    "sel_owner",
                                    state.options.owner.unwrap_or_else(|| "all".into()),
                                ),
                            ]);

                        let builder = builder.query(match &state.options.state {
                            BookingState::Active => &[("active", "on")][..],
                            BookingState::Inactive => &[("inactive", "on")][..],
                            BookingState::All => &[("active", "on"), ("inactive", "on")][..],
                        });

                        let builder = builder
                            .query(&date_filter_to_query("s_from", state.options.start_from));
                        let builder =
                            builder.query(&date_filter_to_query("s_to", state.options.start_to));
                        let builder =
                            builder.query(&date_filter_to_query("e_from", state.options.end_from));
                        let builder =
                            builder.query(&date_filter_to_query("e_to", state.options.end_to));

                        let resp = builder.send().await?;

                        log::debug!("URL: {}", resp.url());

                        let result = parser::parse_query(&resp.text().await?)?;

                        let next_offset = match result.paging {
                            None => None,
                            Some(p) if p.to >= p.total => None,
                            Some(p) => Some(p.to),
                        };

                        let y = stream::iter(result.bookings).map(|b| Ok(b));

                        Ok(Some((
                            y,
                            ListState {
                                offset: next_offset,
                                ..next
                            },
                        )))
                    }
                }
            }
        })
        .try_flatten()
    }
}
