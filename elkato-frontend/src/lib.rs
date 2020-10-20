#![recursion_limit = "512"]

use anyhow::Error;

use patternfly_yew::*;

use elkato_common::data::*;

use headers::authorization::Credentials;
use headers::Authorization;
use wasm_bindgen::prelude::*;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

struct Model {
    link: ComponentLink<Self>,
    ft: Option<FetchTask>,
    bookings: Vec<Booking>,
}

pub enum Msg {
    FetchData(),
    FetchReady(Result<Vec<Booking>, Error>),
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
                self.bookings = response.unwrap_or_default();
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
                        { for self.bookings.iter().map(|booking|{
                            html_nested!{

                        <Card
                            selectable=true
                            title={html_nested!{<>
                                { &booking.start } {" / "} { &booking.end }
                            </>}}
                            >
                            <div>{ &booking.id } {" / "} { &booking.resource }</div>
                            <div>{ booking.description.as_ref().map(|s|s.clone()).unwrap_or_default() }</div>

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
}

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<Model>::new().mount_to_body();
}
