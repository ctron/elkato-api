use anyhow::{anyhow, Error, Result};

use elkato_common::data::Booking;
use patternfly_yew::*;
use yew::prelude::*;
use yew::services::fetch::{Request, *};

use headers::authorization::Credentials;
use headers::Authorization;

use chrono::{Date, DateTime, Duration, TimeZone, Utc};
use chrono_tz::Europe::Berlin;
use url::Url;
use yew::format::{Json, Nothing};

use crate::data::Config;
use crate::BASE_URL;

pub struct CurrentView {
    link: ComponentLink<Self>,
    ft: Option<FetchTask>,
    bookings: Vec<Booking>,
}

pub enum Msg {
    FetchData,
    FetchReady(Result<Vec<Booking>, Error>),
    Open(Option<Url>),
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
                self.bookings = self.select(response.unwrap_or_default());
            }
            Msg::Open(url) => {
                match url {
                    Some(url) => {
                        yew::utils::window()
                            .open_with_url_and_target(url.as_str(), "_blank")
                            .ok();
                    }
                    None => {}
                }
                return false;
            }
        }

        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        let now = Utc::now();

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
                            onclick=self.make_onclick(&sel_booking)
                            selected={sel_booking.is_active(&now)}
                            title={html_nested!{<>
                                { self.title(&sel_booking, &now) }
                            </>}}
                            >
                            <div>{ &sel_booking.resource }</div>
                            { for sel_booking.description.iter().map(|desc|{
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
    fn make_onclick<E>(&self, sel_booking: &Booking) -> Callback<E> {
        let loc = sel_booking.location.clone();
        self.link.callback(move |_| Msg::Open(loc.clone()))
    }

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

    fn title(&self, booking: &Booking, now: &DateTime<Utc>) -> String {
        let dur = booking.end - booking.start;

        let dur = if dur.num_minutes() >= 60 {
            format!("{} h", dur.num_hours())
        } else {
            format!("{} min", dur.num_minutes())
        };

        let start_date = booking.start.date();
        let end_date = booking.start.date();

        let tz = &Berlin;

        if start_date == end_date {
            let date = format_date(&start_date, now, tz);
            format!(
                "{} | {} → {} ({})",
                date,
                booking.start.with_timezone(tz).format("%H:%M"),
                booking.end.with_timezone(tz).format("%H:%M"),
                dur
            )
        } else {
            let start_date = format_date(&start_date, now, tz);
            let end_date = format_date(&end_date, now, tz);
            format!(
                "{} {} → {} {} ({})",
                start_date,
                booking.start.with_timezone(tz).format("%H:%M"),
                end_date,
                booking.end.with_timezone(tz).format("%H:%M"),
                dur
            )
        }
    }

    fn select(&self, mut bookings: Vec<Booking>) -> Vec<Booking> {
        // sort by time asc
        bookings.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| a.end.cmp(&b.end)));

        log::info!("Bookings: {:?}", bookings);

        let now = Utc::now();

        let mut new = Vec::new();

        for b in bookings.iter().rev() {
            if b.end >= now {
                new.push(b.clone());
            } else {
                new.push(b.clone());
                break;
            }
        }

        new.reverse();
        new
    }
}

fn format_date<Tz>(date: &Date<Utc>, now: &DateTime<Utc>, tz: &Tz) -> String
where
    Tz: TimeZone,
    Tz::Offset: std::fmt::Display,
{
    let now = now.date();
    let day = Duration::days(1);

    if &now == date {
        "Today".to_string()
    } else if &(now + day) == date {
        "Tomorrow".to_string()
    } else if &(now - day) == date {
        "Yesterday".to_string()
    } else {
        date.with_timezone(tz).format("%v").to_string()
    }
}
