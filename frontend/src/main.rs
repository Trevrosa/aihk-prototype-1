use chrono::{DateTime, Local};
use frontend::{get_textarea, set_text};
use gloo_net::http::{Request, Response};

use wasm_bindgen_futures::spawn_local;

use yew::{prelude::*, virtual_dom::VNode};
use yew_router::prelude::*;

use serde::Deserialize;

use common::Post;

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
            let submit_post: Callback<MouseEvent> = Callback::from(move |_| {
                let texts: String = get_textarea("input");
                let user: String = get_textarea("user");

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

                    let api_key: String = format!("Bearer {}", common::API_KEY);

                    let req = Request::post("/api/submit_post")
                        .header("authorization", &api_key)
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
                <div class="container">
                    <div class="main">
                        <h1>{ "Posts" }</h1>
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

                        <button onclick={submit_post}>{ "Submit" }</button>
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
    let data: UseStateHandle<Result<Vec<Post>, String>> =
        use_state(|| Err(String::from("not found")));

    {
        let data = data.clone();

        use_effect(|| {
            if data.is_err() {
                spawn_local(async move {
                    let posts = get_api_json::<Vec<Post>>("/api/get_posts").await;

                    log::info!("got {} posts", posts.as_ref().map_or(0, std::vec::Vec::len));
                    data.set(posts);
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
                            <div id={ comment.id.map_or("none".to_string(), |id| id.to_string()) }>
                                {
                                    format!("{}: {}", &comment.username, &comment.content)
                                }
                            </div>
                        }
                    })
                    .collect::<Html>(),
                None => html! {},
            };

            let timestamp: String =
                DateTime::<Local>::from(DateTime::from_timestamp(post.timestamp, 0).unwrap())
                    .format("%d/%m/%Y %H:%M")
                    .to_string();

            html! {
                <div class="post" id={ post.id.map_or("none".to_string(), |id| id.to_string()) }>
                    <div class="post-header">
                        <p class="username">{ format!("{} said:", &post.username) }</p>
                        <p class="timestamp">{ timestamp }</p>
                    </div>
                    <p class="content">{ &post.content }</p>
                    <div class="comments">{ comments }</div>
                </div>
            }
        })
        .collect::<Html>();

    html! {
        { posts }
    }
}

async fn get_api_json<T: for<'a> Deserialize<'a>>(path: &str) -> Result<T, String> {
    let api_key: String = format!("Bearer {}", common::API_KEY);

    let resp: Response = Request::get(path)
        .header("authorization", &api_key)
        .send()
        .await
        .unwrap();

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
