use anyhow::{anyhow, Error, Result};

use elkato_common::data::Booking;
use patternfly_yew::*;
use yew::prelude::*;
use yew::services::fetch::{Request, *};

use headers::authorization::Credentials;
use headers::Authorization;

use chrono::Utc;
use chrono_tz::Europe::Berlin;
use yew::format::{Json, Nothing};

use crate::data::Config;
use crate::BASE_URL;

pub struct CurrentView {
    link: ComponentLink<Self>,
    ft: Option<FetchTask>,
    bookings: Vec<Selected<Booking>>,
}

#[derive(Debug)]
pub struct Selected<T> {
    pub value: T,
    pub selected: bool,
}

impl<T> Selected<T> {
    pub fn new(value: T) -> Self {
        Selected {
            value,
            selected: false,
        }
    }
}

impl<T> Clone for Selected<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Selected {
            selected: self.selected,
            value: self.value.clone(),
        }
    }
}

pub enum Msg {
    FetchData,
    FetchReady(Result<Vec<Booking>, Error>),
}

impl Component for CurrentView {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_message(Msg::FetchData);

        Self {
            link,
            ft: None,
            bookings: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchData => {
                self.ft = match self.fetch_update() {
                    Ok(ft) => Some(ft),
                    Err(err) => {
                        Self::error(err.to_string());
                        None
                    }
                }
            }
            Msg::FetchReady(response) => {
                self.bookings = self.select(
                    response
                        .unwrap_or_default()
                        .iter()
                        .map(|b| Selected::new(b.clone()))
                        .collect(),
                );
            }
        }

        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <>
                <PageSection variant=PageSectionVariant::Light limit_width=true>
                    <Content>
                        <h1>{"Current Bookings"}</h1>
                    </Content>
                </PageSection>
                <PageSection>
                    <Gallery gutter=true>
                        { for self.bookings.iter().map(|sel_booking|{
                            html_nested!{

                        <Card
                            selectable=true
                            selected={sel_booking.selected}
                            title={html_nested!{<>
                                { self.title(&sel_booking.value) }
                            </>}}
                            >
                            <div>{ &sel_booking.value.resource }</div>
                            { for sel_booking.value.description.iter().map(|desc|{
                                html_nested!{
                                    <div>{ desc }</div>
                                }
                            })}

                        </Card>

                            }
                        })}

                    </Gallery>
                </PageSection>
            </>
        }
    }
}

impl CurrentView {
    fn error<S: Into<String>>(err: S) {
        ToastDispatcher::new().toast(Toast {
            title: "Error fetching data".into(),
            r#type: Type::Danger,
            body: html! {
                <p>{err.into()}</p>
            },
            ..Default::default()
        })
    }

    fn fetch_update(&mut self) -> Result<yew::services::fetch::FetchTask> {
        let config =
            Config::load().map_err(|err| anyhow!("Failed to restore user information: {}", err))?;

        if config.user.club.is_empty()
            || config.user.username.is_empty()
            || config.user.password.is_none()
        {
            return Err(anyhow!("Missing user information"));
        }

        let callback = self.link.batch_callback(
            move |response: Response<Json<Result<Vec<Booking>, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                if meta.status.is_success() {
                    vec![Msg::FetchReady(data)]
                } else {
                    vec![] // FIXME: Handle this error accordingly.
                }
            },
        );

        let auth = Authorization::basic(
            &config.user.username,
            config
                .user
                .password
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(""),
        );

        let club = &config.user.club;
        let club = percent_encoding::utf8_percent_encode(club, percent_encoding::NON_ALPHANUMERIC);

        ::log::info!("Auth: {:?}", auth.0.encode());

        let request = Request::get(format!("{}/{}/bookings/current", BASE_URL.to_owned(), club))
            .header("Authorization", auth.0.encode())
            .body(Nothing);

        log::info!("Request: {:?}", request);

        let request = request.unwrap();

        let ft = FetchService::fetch(request, callback);

        log::info!("FT: {:?}", ft);

        ft
    }

    fn title(&self, booking: &Booking) -> String {
        format!(
            "{} -> {}",
            booking
                .start
                .with_timezone(&Berlin)
                .format("%d-%m-%Y %H:%M"),
            booking.end.with_timezone(&Berlin).format("%d-%m-%Y %H:%M")
        )
    }

    fn select(&self, mut bookings: Vec<Selected<Booking>>) -> Vec<Selected<Booking>> {
        // sort by time asc

        bookings.sort_by(|a, b| {
            a.value
                .start
                .cmp(&b.value.start)
                .then_with(|| a.value.end.cmp(&b.value.end))
        });

        log::info!("Bookings: {:?}", bookings);

        let now = Utc::now();
        let mut selected: Option<&String> = None;

        // iterate: back to front
        for b in bookings.iter().rev() {
            log::info!("Sel: {:?}", b);
            if b.value.start > now {
                selected = Some(&b.value.id);
            } else {
                if selected.is_none() {
                    selected = Some(&b.value.id);
                }
                break;
            }
        }

        let selected = selected.map(|s| s.clone());

        // if an item is selected...
        if let Some(sel) = selected {
            // ...mark as selected
            for ref mut b in &mut bookings {
                if &b.value.id == &sel {
                    b.selected = true;
                }
            }

            let mut new = Vec::new();

            for b in bookings.iter().rev() {
                let mut b = b.clone();

                if b.value.id.eq(&sel) {
                    b.selected = true;
                    new.push(b);
                    new.reverse();
                    return new;
                } else {
                    new.push(b);
                }
            }
            new
        } else {
            bookings
        }
    }
}
