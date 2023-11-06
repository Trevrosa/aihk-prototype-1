use chrono::{DateTime, Local, Utc};
use frontend::{get_document, get_textarea, set_text};
use gloo_net::http::{Request, Response};

use gloo_storage::{Storage, LocalStorage};
use rustrict::CensorStr;
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

const EMPTY_STAR_NAMES: &[&str] = &[
    "/assets/star-empty-1.png",
    "/assets/star-empty-2.png",
    "/assets/star-empty-3.png",
    "/assets/star-empty-4.png",
    "/assets/star-empty-5.png",
    "/assets/star-empty-6.png",
    "/assets/star-empty-7.png",
    "/assets/star-empty-8.png",
    "/assets/star-empty-9.png",
    "/assets/star-empty-10.png",
];

const FILLED_STAR_NAMES: &[&str] = &[
    "/assets/star-filled-1.png",
    "/assets/star-filled-2.png",
    "/assets/star-filled-3.png",
    "/assets/star-filled-4.png",
    "/assets/star-filled-5.png",
    "/assets/star-filled-6.png",
    "/assets/star-filled-7.png",
    "/assets/star-filled-8.png",
    "/assets/star-filled-9.png",
    "/assets/star-filled-10.png",
];

#[allow(clippy::needless_pass_by_value)]
fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => {
            let get_new_posts: Callback<MouseEvent> = Callback::from(move |_| {
                let posts_element = get_document().get_element_by_id("posts").unwrap();

                let stars = get_document().get_elements_by_class_name("star");

                spawn_local(async move {
                    let posts: Vec<Post> =
                        match get_api_json::<Option<Vec<Post>>>("/api/get_posts").await {
                            Ok(Some(post)) => post,
                            _ => panic!(),
                        };

                    let num_posts = posts.len();
                    log::info!("got {num_posts} posts");

                    let posts: String = posts
                        .iter()
                        .map(|post| {
                            let mut comments = match post.comments.as_ref() {
                                Some(comments) => comments
                                    .iter()
                                    .map(|comment| {
                                        format!(r#"<div id={}>{}: {}</div>"#,
                                            comment.id.to_string(),
                                            &comment.username,
                                            &comment.content.censor(),
                                        )
                                    })
                                    .collect::<Vec<String>>()
                                    .join("\n"),
                                None => "".to_string(),
                            };

                            comments.push_str("\n");

                            let timestamp: DateTime<Utc> = DateTime::from_timestamp(post.created, 0).unwrap();
                            let timestamp: String =
                                DateTime::<Local>::from(timestamp)
                                    .format("%d/%m/%Y %H:%M")
                                    .to_string();

                            format!(
                                r#"
                                <div class="border border-2 rounded border-primary-subtle position-absolute top-50 bg-dark col-3 p-2 px-10" id={} style="visibility: hidden;">
                                    <div class="d-flex flex-row">
                                        <p class="flex-grow-1">{} said:</p>
                                        <p>{timestamp}</p>
                                    </div>

                                    <div class="d-flex flex-row">
                                        <p class="flex-grow-1">{}</p>
                                        <a class="mb-3" href="javascript:hide({})">{}</a>
                                    </div>

                                    <div class="fst-italic text-wrap border-top pt-3 fs-6">{comments}</div>
                                </div>"#,
                                post.id.to_string(),
                                &post.username,
                                &post.content.censor(),
                                post.id,
                                "close"
                            )
                        })
                        .collect::<Vec<String>>()
                        .join("\n");

                    posts_element.set_inner_html(&posts);

                    for i in 0..10 {
                        if i+1 > num_posts {
                            stars
                                .get_with_index(i as u32)
                                .unwrap()
                                .set_attribute("src", EMPTY_STAR_NAMES[i])
                                .unwrap();
                            continue;
                        }
                        stars
                            .get_with_index(i as u32)
                            .unwrap()
                            .set_attribute("src", FILLED_STAR_NAMES[i])
                            .unwrap();
                    }
                });
            });

            let log_in: Callback<MouseEvent> = Callback::from(move |_| {
                let username = get_textarea("inputUsername");
                let password = get_textarea("inputPassword");

                spawn_local(async {
                    match post_api_json::<Option<String>, _>("/api/login", &User::new(username, password)).await {
                        Ok(Some(session)) => {
                            set_text("a", "logged in");
                            LocalStorage::set("session", session).unwrap()
                        },
                        Ok(None) => set_text("a", "no session"),
                        Err(_) => set_text("a", "failed to send request"),
                    };
                });
            });

            let create_account: Callback<MouseEvent> = Callback::from(move |_| {
                let username = get_textarea("inputUsername");
                let password = get_textarea("inputPassword");

                spawn_local(async {
                    match post_api_json::<Option<String>, _>("/api/create_account", &User::new(username, password)).await {
                        Ok(Some(session)) => {
                            set_text("a", "created!");
                            LocalStorage::set("session", session).unwrap()
                        },
                        Ok(None) => set_text("a", "no session"),
                        Err(_) => set_text("a", "failed to send request"),
                    };
                });
            });

            html! {
                <>

                <div class="row vw-100 vh-100">
                    <div class="col"/>
                    <div class="col">
                        <img src="/assets/tree.jpg" class="vh-100 mx-auto d-block" alt="A banyan tree."/>
                    </div>
                    <div class="col pt-2">
                        <h1 class="mb-3 text-center">{ "Manage account" }</h1>
                        
                        <div class="border rounded p-2 mb-2">
                            <div class="mb-3">
                                <label for="inputUsername" class="form-label">{ "Username" }</label>
                                <input type="text" placeholder="Type here" class="form-control" id="inputUsername" aria-describedby="usernameInfo"/>
                                <div id="usernameInfo" class="form-text">{ "Usernames are unique!" }</div>
                            </div>
                            <div class="mb-3">
                                <label for="inputPassword" class="form-label">{ "Password" }</label>
                                <input class="form-control" id="inputPassword" placeholder="Type here"/>
                            </div>
                            <button onclick={log_in} class="btn btn-primary">{ "Log in!" }</button>
                            <button onclick={create_account} class="btn btn-primary ms-2">{ "Create account!" }</button>
                        </div>

                        <div class="border rounded p-2">
                            <p class="text-center">{ "Options" }</p>
                            <div class="mb-3 form-check">
                                <input type="checkbox" class="form-check-input" id="uncensor" name="uncensor"/>
                                <label for="uncensor" class="form-check-label">{ "Don't censor innapropriate words" }</label>
                            </div>

                            <button onclick={save_options} class="btn btn-primary">{ "Apply settings" }</button>
                        </div>
                    </div>
                </div>

                <a href="#">
                    <img onclick={get_new_posts} src="/assets/reload.png" style="position: absolute; top: -13.4%; right: 23%; transform: scale(0.28);" alt="Reload button."/>
                </a>

                <a href="javascript:show(\"1\");">
                    <img src="/assets/star-empty-1.png" class="star" style="position: absolute; top: 39.6%; left: 57%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"2\");">
                    <img src="/assets/star-empty-2.png" class="star" style="position: absolute; top: 42.7%; left: 50%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"3\");">
                    <img src="/assets/star-empty-3.png" class="star" style="position: absolute; top: 30%; left: 46%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"4\");">
                    <img src="/assets/star-empty-4.png" class="star" style="position: absolute; top: 14%; left: 37.9%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"5\");">
                    <img src="/assets/star-empty-5.png" class="star" style="position: absolute; top: 29.3%; left: 53.5%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"6\");">
                    <img src="/assets/star-empty-6.png" class="star" style="position: absolute; top: 15%; left: 45%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"7\");">
                    <img src="/assets/star-empty-7.png" class="star" style="position: absolute; top: 44.4%; left: 31.5%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"8\");">
                    <img src="/assets/star-empty-8.png" class="star" style="position: absolute; top: 36%; left: 34.3%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"9\");">
                    <img src="/assets/star-empty-9.png" class="star" style="position: absolute; top: 25%; left: 38%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"10\");">
                    <img src="/assets/star-empty-10.png" class="star" style="position: absolute; top: 43.3%; left: 40%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <div class="z-3 d-flex justify-content-center align-items-center" id="posts"/>

                </>
            }
        }
        Route::NotFound => html! {
            <h1 class="text-center">{ "404 Not Found" }</h1>
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

async fn get_api_json<T>(path: &str) -> Result<T, String>
where
    T: for<'a> Deserialize<'a>,
{
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

async fn get_api_json_bearing<T>(path: &str, auth: &str) -> Result<T, String>
where
    T: for<'a> Deserialize<'a>,
{
    let resp: Response = Request::get(path)
        .header("authorization", &format!("Bearer {auth}"))
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

async fn _get_api_bearing(path: &str, auth: &str) -> Result<String, String> {
    let resp: Response = Request::get(path)
        .header("authorization", &format!("Bearer {auth}"))
        .send()
        .await
        .unwrap();

    let resp: Result<String, String> = if resp.ok() {
        resp.text().await.map_err(|err| err.to_string())
    } else {
        Err(format!(
            "Error fetching data {} ({})",
            resp.status(),
            resp.status_text()
        ))
    };

    resp
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
