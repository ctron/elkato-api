use crate::data::Booking;
use crate::parser;
use anyhow::Context;
use futures::stream::Iter;
use futures::{stream, StreamExt, TryFuture, TryFutureExt, TryStreamExt};
use futures::{Stream, TryStream};
use reqwest::header::HeaderValue;
use reqwest::{header, Url};

pub struct Client {
    config: Config,
    client: reqwest::Client,
}

pub struct Config {
    pub url: Url,
}

#[derive(Clone, Debug)]
pub struct User {
    pub club: String,
    pub username: String,
    pub password: String,
}

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
    ) -> impl TryStream<Item = Result<Booking, anyhow::Error>> + '_ {
        let url = self.config.url.join("/buchung/search.php");
        let client = self.client.clone();

        stream::try_unfold((Some(0usize), client, user, url), move |_state| {
            async {
                let state = _state;
                let offset = state.0.clone();
                let client = state.1.clone();
                let user = state.2.clone();
                let url = state.3.clone();

                match offset {
                    // having no offset means, we finish up in the last iteration
                    None => Result::<_, anyhow::Error>::Ok(None),
                    // having an offset means we need to pull in more data
                    Some(offset) => {
                        let resp = client
                            .get(url.clone()?)
                            .basic_auth(user.username.clone(), Some(user.password.clone()))
                            .query(&[
                                ("club", user.club.clone()),
                                ("search_pos", format!("{}", offset)),
                                ("sel_room", "all".into()),
                                ("sel_booker", "all".into()),
                                ("sel_owner", "all".into()),
                                ("active", "on".into()),
                            ])
                            .send()
                            .await?;

                        let result = parser::parse_query(&resp.text().await?)?;

                        println!("List: {:x?}", result);

                        let next = match result.paging {
                            None => None,
                            Some(p) if p.to >= p.total => None,
                            Some(p) => Some(p.to),
                        };

                        let y: Vec<Result<Booking, anyhow::Error>> =
                            result.bookings.iter().map(|b| Ok(b.clone())).collect();

                        Ok(Some((y, (next, client.clone(), user.clone(), url.clone()))))
                    }
                }
            }
        })
        .map_ok(futures::stream::iter)
        .try_flatten()

        //Ok(stream.map_ok(futures::stream::iter).try_flatten())
        //stream.try_flatten()
        //stream.try_flatten()

        //         self.list_bookings_vec(user).map(|v| v.iter()).try_flatten()

        /*
        let resp = self
            .client
            .get(self.config.url.join("/buchung/search.php")?)
            .basic_auth(user.username, Some(user.password))
            .query(&[
                ("club", user.club.as_str()),
                ("search_pos", start.as_str()),
                ("sel_room", "all"),
                ("sel_booker", "all"),
                ("sel_owner", "all"),
                ("active", "on"),
            ])
            .send()
            .await?;

        let result = parser::parse_query(&resp.text().await?)?;

        Ok(())
        */
    }

    /*
    fn list_bookings_vec(
        &self,
        user: User,
    ) -> impl Stream<Item = anyhow::Result<Vec<Booking>>> + '_ {
        let stream = stream::try_unfold(Some(0usize), |offset| async move {
            match offset {
                // having no offset means, we finish up in the last iteration
                None => Ok(None),
                // having an offset means we need to pull in more data
                Some(offset) => {
                    let resp = self
                        .client
                        .get(self.config.url.join("/buchung/search.php")?)
                        .basic_auth(user.username, Some(user.password))
                        .query(&[
                            ("club", user.club.as_str()),
                            ("search_pos", &format!("{}", offset)),
                            ("sel_room", "all"),
                            ("sel_booker", "all"),
                            ("sel_owner", "all"),
                            ("active", "on"),
                        ])
                        .send()
                        .await?;

                    let result = parser::parse_query(&resp.text().await?)?;

                    let r = match result.paging {
                        None => Ok(Some((result.bookings, None))),
                        Some(p) if p.to >= p.total => Ok(Some((result.bookings, None))),
                        Some(p) => Ok(Some((result.bookings, Some(p.to)))),
                    };

                    r
                }
            }
        });

        stream
    }
     */
}
