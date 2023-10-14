use gloo_net::http::{Request, Response};

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, HtmlTextAreaElement};

use yew::prelude::*;
use yew_router::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[allow(clippy::needless_pass_by_value)]
fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => {
            let submit: Callback<MouseEvent> = Callback::from(move |_| {
                let texts = get_textarea("input");
                let user = get_textarea("user");

                log::info!("{user}: {texts}");

                spawn_local(async move {
                    let request_body: Post = Post::new(user, texts);

                    let req = Request::post("/api/submit_post")
                        .json(&request_body)
                        .unwrap()
                        .send()
                        .await;

                    if req.is_ok() {
                        log::info!("Success");
                    } else {
                        log::info!("Failed");
                    }
                });
            });

            html! {
                <div class="outside">
                    <div class="main">
                        <h1>{ "HI" }</h1>
                        <Posts/>
                    </div>
                    <div class="inputing">
                        <h2>{ "Input" }</h2>

                        <p>{ "username" }</p>
                        <textarea id="user"/>

                        <p>{ "content" }</p>
                        <textarea id="input"/>

                        <button onclick={submit}>{ "Submit" }</button>
                    </div>
                </div>
            }
        }
        Route::NotFound => html! {
            <h1 style="text-align: center;">{ "404 Not Found" }</h1>
        },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct Post {
    username: String,
    content: String,
}

impl Post {
    fn new(username: String, content: String) -> Self {
        Self { username, content }
    }
}

impl std::fmt::Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} said: {}", self.username, self.content)
    }
}

fn get_textarea(id: &str) -> String {
    window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id(id)
        .unwrap()
        .unchecked_into::<HtmlTextAreaElement>()
        .value()
}

#[function_component(Posts)]
fn show_posts() -> Html {
    let data = use_state_eq(|| None);

    {
        let data = data.clone();

        use_effect(|| {
            spawn_local(async move {
                log::info!("got posts");
                data.set(Some(get_api_json::<Vec<Post>>("/api/get_posts").await));
            });
        });
    }

    let posts: Vec<Post> = if let Some(Ok(data)) = &*data {
        data.clone()
    } else {
        vec![Post::new("loading".to_string(), String::new())]
    };

    html! {
        {
            posts
                .iter()
                .map(|hello| html!
                    {
                        <p>{hello}</p>
                    }
            ).collect::<Html>()
        }
    }
}

async fn get_api_json<T: for<'a> Deserialize<'a>>(path: &str) -> Result<T, String> {
    let resp: Response = Request::get(path).send().await.unwrap();

    let resp: Result<T, String> = if resp.ok() {
        resp.json::<T>().await.map_err(|err| err.to_string())
    } else {
        Err(format!(
            "Error fetching data {} ({})",
            resp.status(),
            resp.status_text()
        ))
    };

    resp
}

async fn _get_api(path: &str) -> String {
    let resp: Response = Request::get(path).send().await.unwrap();

    let resp: Result<String, String> = if resp.ok() {
        resp.text().await.map_err(|err| err.to_string())
    } else {
        Err(format!(
            "Error fetching data {} ({})",
            resp.status(),
            resp.status_text()
        ))
    };

    match Some(resp) {
        None => String::from("not found"),
        Some(Ok(data)) => data,
        Some(Err(err)) => err,
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
