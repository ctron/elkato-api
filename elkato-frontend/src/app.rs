use crate::current::CurrentView;
use crate::data::Config;

use anyhow::anyhow;

use patternfly_yew::*;

use elkato_client::User;
use std::collections::HashMap;
use url::Url;
use yew::prelude::*;
use yew::services::storage::*;

pub struct App {}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let loc = yew::utils::window().location();
        let url = Url::parse(&loc.href().unwrap()).unwrap();

        let q: HashMap<_, _> = url.query_pairs().collect();

        match (q.get("club"), q.get("username"), q.get("password")) {
            (Some(club), Some(username), Some(password)) => {
                let config = Config {
                    user: User {
                        club: club.to_string(),
                        username: username.to_string(),
                        password: Some(password.to_string()),
                    },
                };
                config.store().ok();
                // clear out query string
                loc.set_search("").ok();
            }
            _ => {}
        }

        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let tools = html! {};

        html! {
        <>
            <ToastViewer/>
            <Page
                tools=tools
                >
                    <CurrentView/>
            </Page>
        </>
        }
    }
}
