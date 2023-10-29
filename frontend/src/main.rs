use chrono::{DateTime, Local};
use frontend::{get_textarea, set_text};
use gloo_net::http::{Request, Response};

use gloo_storage::Storage;
use wasm_bindgen_futures::spawn_local;

use yew::{prelude::*, virtual_dom::VNode};
use yew_router::prelude::*;

use serde::{Deserialize, Serialize};

use common::{Post, User};

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

                spawn_local(async move {
                    if texts.is_empty() {
                        set_text("status2", "cannot be empty");
                        return;
                    }

                    let session_id =
                        if let Ok(id) = gloo_storage::LocalStorage::get::<String>("session") {
                            format!("Bearer {id}")
                        } else {
                            set_text("status2", "not signed in");
                            return;
                        };

                    log::info!("{}", &session_id);

                    let req = Request::post("/api/submit_post")
                        .header("authorization", &session_id)
                        .body(texts)
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

                        set_text("status2", &resp);
                    }
                });
            });

            let login: Callback<MouseEvent> = Callback::from(move |_| {
                let username: String = get_textarea("user");
                let password: String = get_textarea("pass");

                let request: User = User::new(username, password);

                spawn_local(async move {
                    let resp = post_api_json::<String, _>("/api/login", &request).await;

                    match resp {
                        Ok(session) => {
                            gloo_storage::LocalStorage::set("session", &session).unwrap();
                            let sta = format!("got session: {session}");

                            set_text("status1", &sta);
                        }
                        Err(err) => set_text("status1", &err),
                    };
                });
            });

            let signup: Callback<MouseEvent> = Callback::from(move |_| {
                let username: String = get_textarea("user");
                let password: String = get_textarea("pass");

                let request: User = User::new(username, password);

                spawn_local(async move {
                    let resp = post_api_json::<String, _>("/api/create_account", &request).await;

                    match resp {
                        Ok(session) => gloo_storage::LocalStorage::set("session", session).unwrap(),
                        Err(err) => set_text("status1", &err),
                    };
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
                        <h1>{ "Input" }</h1>

                        <h2>{ "Login" }</h2>

                        <p>{ "username" }</p>
                        <textarea id="user"/>

                        <p>{ "password" }</p>
                        <textarea id="pass"/>

                        <button onclick={login}>{ "Log in" }</button>
                        <button onclick={signup}>{ "Create account" }</button>
                        <p class="statuses" id="status1"/>

                        <h2>{ "Post" }</h2>

                        <Signed/>

                        <p>{ "content" }</p>
                        <textarea id="input"/>

                        <button onclick={submit_post}>{ "Submit" }</button>
                        <p class="statuses" id="status2"/>
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

#[function_component(Signed)]
fn signed_in() -> Html {
    let data = use_state(|| None);

    {
        let data = data.clone();

        use_effect(move || {
            if data.is_none() {
                if gloo_storage::LocalStorage::get::<String>("session").is_ok() {
                    data.set(Some("signed in"));
                } else {
                    data.set(Some("not signed in"));
                }
            }
        });
    }

    let signed = if let Some(data) = &*data {
        data
    } else {
        "loading"
    };

    html! {
        <h2>{ signed }</h2>
    }
}

#[function_component(Posts)]
fn show_posts() -> Html {
    let data: UseStateHandle<Option<Vec<Post>>> = use_state(|| None);

    {
        let data: UseStateHandle<Option<Vec<Post>>> = data.clone();

        use_effect(|| {
            if data.is_none() {
                spawn_local(async move {
                    let posts = get_api_json::<Option<Vec<Post>>>("/api/get_posts").await;

                    log::info!(
                        "got {} posts",
                        match posts.as_ref() {
                            Ok(res) => match res {
                                Some(posts) => posts.len(),
                                None => 0,
                            },
                            Err(_) => 0,
                        }
                    );

                    let posts = match posts {
                        Ok(posts) => posts,
                        Err(_) => None,
                    };

                    data.set(posts);
                });
            }
        });
    }

    let posts: Vec<Post> = if let Some(data) = &*data {
        data.clone()
    } else {
        return html! {};
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
                            <div id={ comment.id.to_string() }>
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
                DateTime::<Local>::from(DateTime::from_timestamp(post.created, 0).unwrap())
                    .format("%d/%m/%Y %H:%M")
                    .to_string();

            html! {
                <div class="post" id={ post.id.to_string() }>
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

async fn post_api_json<T, J>(path: &str, json: &J) -> Result<T, String>
where
    T: for<'a> Deserialize<'a>,
    J: Serialize,
{
    let resp: Response = Request::post(path)
        .json(json)
        .unwrap()
        .send()
        .await
        .unwrap();

    let resp: Result<T, String> = if resp.ok() {
        resp.json::<T>().await.map_err(|err| err.to_string())
    } else {
        Err(format!(
            "Error fetching data {} ({}): \n{}",
            resp.status(),
            resp.status_text(),
            resp.text().await.unwrap(),
        ))
    };

    resp
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
