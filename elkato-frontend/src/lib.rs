#![recursion_limit = "512"]

use anyhow::Error;

use patternfly_yew::*;

use elkato_common::data::*;

use headers::authorization::Credentials;
use headers::Authorization;

use wasm_bindgen::prelude::*;

use chrono_tz::Europe::Berlin;

use chrono::Utc;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

struct Model {
    link: ComponentLink<Self>,
    ft: Option<FetchTask>,
    bookings: Vec<Selected<Booking>>,
}

pub enum Msg {
    FetchData(),
    FetchReady(Result<Vec<Booking>, Error>),
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

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        log::info!("Here");

        link.send_message(Msg::FetchData());

        Self {
            link,
            ft: None,
            bookings: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchData() => self.ft = Some(self.fetch_update()),
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
        false
    }

    fn view(&self) -> Html {
        html! {
            <Page>
                <PageSection variant=PageSectionVariant::Light limit_width=true>
                    <Content>
                        <h1>{"Next Bookings"}</h1>
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
            </Page>
        }
    }
}

impl Model {
    fn fetch_update(&mut self) -> yew::services::fetch::FetchTask {
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

        let auth = Authorization::basic("demo", "demo");

        log::info!("Auth: {:?}", auth.0.encode());

        let request = Request::get(
            "https://elkato-elkato.apps.wonderful.iot-playground.org/demo/bookings/current",
        )
        .header("Authorization", auth.0.encode())
        .body(Nothing);

        log::info!("Request: {:?}", request);

        let request = request.unwrap();

        let ft = FetchService::fetch(request, callback);

        log::info!("FT: {:?}", ft);

        ft.unwrap()
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

        // back to front

        log::info!("Bookings: {:?}", bookings);

        let now = Utc::now();
        let mut selected: Option<&String> = None;

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

        if let Some(sel) = selected {
            for ref mut b in &mut bookings {
                if b.value.id.eq(&sel) {
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

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<Model>::new().mount_to_body();
}
