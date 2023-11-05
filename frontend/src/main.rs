use chrono::{DateTime, Local, Utc};
use frontend::{get_document, get_textarea, set_text};
use gloo_net::http::{Request, Response};

use gloo_storage::Storage;
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
                            let comments = match post.comments.as_ref() {
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

                            let timestamp: DateTime<Utc> = DateTime::from_timestamp(post.created, 0).unwrap();
                            let timestamp: String =
                                DateTime::<Local>::from(timestamp)
                                    .format("%d/%m/%Y %H:%M")
                                    .to_string();

                            format!(
                                r#"
                                <div class="border border-2 position-absolute top-50 bg-dark col-3 p-2 px-10" id={} style="visibility: hidden;">
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

                    for i in 0..3 { // TODO: change to 10
                        if i > num_posts {
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

            html! {
                <>

                <img src="/assets/tree.jpg" class="vh-100 mx-auto d-block" alt="A banyan tree."/>
                <a href="#">
                    <img onclick={get_new_posts} src="/assets/reload.png" style="position: absolute; top: -5%; right: 25%; transform: scale(0.5);" alt="Reload button."/>
                </a>

                <a href="javascript:show(\"1\");">
                    <img src="/assets/star-empty-1.png" class="star" style="position: absolute; top: 45%; left: 60%; transform: scale(0.8);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"2\");">
                    <img src="/assets/star-empty-2.png" class="star" style="position: absolute; top: 40%; left: 50%; transform: scale(0.8);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"3\");">
                    <img src="/assets/star-empty-3.png" class="star" style="position: absolute; top: 25%; left: 40%; transform: scale(0.8);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"4\");">
                    <img src="/assets/star-empty-4.png" class="star" style="position: absolute; top: 10%; left: 35%; transform: scale(0.8);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"5\");">
                    <img src="/assets/star-empty-5.png" class="star" style="position: absolute; top: 50%; left: 50%; transform: scale(0.8);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"6\");">
                    <img src="/assets/star-empty-6.png" class="star" style="position: absolute; top: 50%; left: 50%; transform: scale(0.8);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"7\");">
                    <img src="/assets/star-empty-7.png" class="star" style="position: absolute; top: 50%; left: 50%; transform: scale(0.8);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"8\");">
                    <img src="/assets/star-empty-8.png" class="star" style="position: absolute; top: 50%; left: 50%; transform: scale(0.8);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"9\");">
                    <img src="/assets/star-empty-9.png" class="star" style="position: absolute; top: 50%; left: 50%; transform: scale(0.8);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"10\");">
                    <img src="/assets/star-empty-10.png" class="star" style="position: absolute; top: 50%; left: 50%; transform: scale(0.8);" alt="A post in the form of a star."/>
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
