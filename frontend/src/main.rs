use frontend::{get_textarea, set_text, Post};
use gloo_net::http::{Request, Response};

use wasm_bindgen_futures::spawn_local;

use yew::{prelude::*, virtual_dom::VNode};
use yew_router::prelude::*;

use serde::Deserialize;

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

                spawn_local(async move {
                    if user.is_empty() || texts.is_empty() {
                        set_text("status", "cannot be empty");
                        return;
                    }

                    let request_body: Post = Post::new(user, texts);
                    log::info!(
                        "sent payload: {}",
                        serde_json::to_string(&request_body).unwrap()
                    );

                    let req = Request::post("/api/submit_post")
                        .json(&request_body)
                        .unwrap()
                        .send()
                        .await;

                    if let Ok(req) = req {
                        let resp = req.text().await.unwrap();

                        if req.ok() {
                            log::info!("Success: {:?}", resp);
                        } else {
                            log::info!("Failed: {:?}", resp);
                        }

                        set_text("status", &resp);
                    }
                });
            });

            html! {
                <div class="outside">
                    <div class="main">
                        <h1>{ "HI" }</h1>
                        <div class="posts">
                            <Posts/>
                        </div>
                    </div>
                    <div class="inputing">
                        <h2>{ "Input" }</h2>

                        <p>{ "username" }</p>
                        <textarea id="user"/>

                        <p>{ "content" }</p>
                        <textarea id="input"/>

                        <button onclick={submit}>{ "Submit" }</button>
                        <p id="status"/>
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

#[function_component(Posts)]
fn show_posts() -> Html {
    let data = use_state(|| Err(String::from("not found")));

    {
        let data = data.clone();

        use_effect(|| {
            if data.is_err() {
                spawn_local(async move {
                    data.set(get_api_json::<Vec<Post>>("/api/get_posts").await);
                    log::info!("got posts {:#?}", &data);
                });
            }
        });
    }

    let posts: Vec<Post> = if let Ok(data) = &*data {
        data.clone()
    } else {
        vec![Post::default()]
    };

    let posts: VNode = posts
        .iter()
        .map(|post| {
            let comments = post.comments.as_ref();

            let comments = match comments {
                Some(comments) => comments
                    .iter()
                    .map(|comment| {
                        html! {
                            <div>{ format!("{}: {}", &comment.username, &comment.content) }</div>
                        }
                    })
                    .collect::<Html>(),
                None => html! {},
            };

            html! {
                <div>
                    <p class="username">{ format!("{} said:", &post.username) }</p>
                    <p class="content">{ &post.content }</p>

                    <div class="comments">
                        { comments }
                    </div>
                </div>
            }
        })
        .collect::<Html>();

    html! {
        { posts }
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
