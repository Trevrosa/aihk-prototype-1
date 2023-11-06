use chrono::{DateTime, Local, Utc};
use frontend::{get_document, get_input, set_text, set_text_str};
use gloo_net::http::{Request, Response};

use gloo_storage::{LocalStorage, SessionStorage, Storage};
use gloo_timers::future::TimeoutFuture;
use rustrict::CensorStr;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use serde::Deserialize;

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

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => {
            SessionStorage::delete("opened");

            // wait for elements to be loaded, then fetch posts and set settings state
            spawn_local(async {
                loop {
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if let Some(check) = document.get_element_by_id("uncensor") {
                                if let Ok(check) = check.dyn_into::<HtmlInputElement>() {
                                    // change setting states
                                    {
                                        if LocalStorage::get::<bool>("censor").is_ok() {
                                            check.set_checked(true);
                                        }
                                    }

                                    // change login status
                                    {
                                        if let Ok(session) =
                                            gloo_storage::LocalStorage::get::<String>("session")
                                        {
                                            if let Ok(Some(username)) =
                                                get_api_json_bearing::<Option<String>>(
                                                    "/api/validate_session",
                                                    &session,
                                                )
                                                .await
                                            {
                                                set_text(
                                                    "login-status",
                                                    format!("signed in as {username}"),
                                                );
                                            } else {
                                                set_text_str(
                                                    "login-status",
                                                    "session invalid, log in again",
                                                );
                                            }
                                        } else {
                                            set_text_str("login-status", "not signed in");
                                        }
                                    }

                                    // fetch posts
                                    {
                                        let posts_element =
                                            document.get_element_by_id("posts").unwrap();
                                        let stars = document.get_elements_by_class_name("star");

                                        let should_censor =
                                            LocalStorage::get::<bool>("censor").is_err();
                                        log::debug!("censor? {should_censor:?}");

                                        // fetch posts on load
                                        spawn_local(async move {
                                            let posts: Vec<Post> =
                                                match get_api_json::<Option<Vec<Post>>>(
                                                    "/api/get_posts",
                                                )
                                                .await
                                                {
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
                                                                let content = if should_censor {
                                                                    comment.content.censor()
                                                                } else {
                                                                    comment.content.clone()
                                                                };

                                                                format!(r#"<div class="" id=comment-{}>{}: {}</div>"#,
                                                                    comment.id,
                                                                    &comment.username,
                                                                    content,
                                                                )
                                                            })
                                                            .collect::<Vec<String>>()
                                                            .join("\n"),
                                                        None => String::new(),
                                                    };

                                                    comments.push_str(&format!(r#"
                                                        <div class="d-flex flex-row">
                                                            <input type="text" id="comment-on-{}" class="flex-grow-1" placeholder="Post a comment"/>
                                                            <a href="">post</a>
                                                        </div>"#,
                                                        post.id
                                                    ));

                                                    let timestamp: DateTime<Utc> = DateTime::from_timestamp(post.created, 0).unwrap();
                                                    let timestamp: String =
                                                        DateTime::<Local>::from(timestamp)
                                                            .format("%d/%m/%Y %H:%M")
                                                            .to_string();

                                                    let content = if should_censor {
                                                        post.content.censor()
                                                    } else {
                                                        post.content.clone()
                                                    };

                                                    format!(
                                                        r#"
                                                        <div class="border border-2 rounded border-primary-subtle position-absolute top-50 bg-dark col-3 p-2 px-10" id="post-{}" style="visibility: hidden;">
                                                            <div class="d-flex flex-row">
                                                                <p class="flex-grow-1">{} said:</p>
                                                                <p>{timestamp}</p>
                                                            </div>
                        
                                                            <div class="d-flex flex-row">
                                                                <p class="flex-grow-1">{}</p>
                                                                <a class="mb-3" href="javascript:hide('post-{}')">{}</a>
                                                            </div>
                        
                                                            <div class="fst-italic text-wrap border-top pt-3 fs-6">{comments}</div>
                                                        </div>"#,
                                                        post.id,
                                                        &post.username,
                                                        content,
                                                        post.id,
                                                        "close"
                                                    )
                                                })
                                                .collect::<Vec<String>>()
                                                .join("\n");

                                            posts_element.set_inner_html(&posts);

                                            for i in 0..10 {
                                                if i + 1 > num_posts {
                                                    stars
                                                        .get_with_index(u32::try_from(i).unwrap())
                                                        .unwrap()
                                                        .set_attribute("src", EMPTY_STAR_NAMES[i])
                                                        .unwrap();
                                                    continue;
                                                }
                                                stars
                                                    .get_with_index(u32::try_from(i).unwrap())
                                                    .unwrap()
                                                    .set_attribute("src", FILLED_STAR_NAMES[i])
                                                    .unwrap();
                                            }
                                        });
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    TimeoutFuture::new(123).await;
                }
            });

            let get_new_posts: Callback<MouseEvent> = Callback::from(move |_| {
                let posts_element = get_document().get_element_by_id("posts").unwrap();

                let stars = get_document().get_elements_by_class_name("star");

                let should_censor = LocalStorage::get::<bool>("censor").is_err();
                log::debug!("censor? {should_censor:?}");

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
                                        let content = if should_censor {
                                            comment.content.censor()
                                        } else {
                                            comment.content.clone()
                                        };

                                        format!(r#"<div id={}>{}: {}</div>"#,
                                            comment.id,
                                            &comment.username,
                                            content,
                                        )
                                    })
                                    .collect::<Vec<String>>()
                                    .join("\n"),
                                None => String::new(),
                            };

                            comments.push_str("\n");

                            let timestamp: DateTime<Utc> = DateTime::from_timestamp(post.created, 0).unwrap();
                            let timestamp: String =
                                DateTime::<Local>::from(timestamp)
                                    .format("%d/%m/%Y %H:%M")
                                    .to_string();

                            let content = if should_censor {
                                post.content.censor()
                            } else {
                                post.content.clone()
                            };

                            format!(
                                r#"
                                <div class="border border-2 rounded border-primary-subtle position-absolute top-50 bg-dark col-3 p-2 px-10" id="post-{}" style="visibility: hidden;">
                                    <div class="d-flex flex-row">
                                        <p class="flex-grow-1">{} said:</p>
                                        <p>{timestamp}</p>
                                    </div>

                                    <div class="d-flex flex-row">
                                        <p class="flex-grow-1">{}</p>
                                        <a class="mb-3" href="javascript:hide('post-{}')">{}</a>
                                    </div>

                                    <div class="fst-italic text-wrap border-top pt-3 fs-6">{comments}</div>
                                </div>"#,
                                post.id,
                                &post.username,
                                content,
                                post.id,
                                "close"
                            )
                        })
                        .collect::<Vec<String>>()
                        .join("\n");

                    posts_element.set_inner_html(&posts);

                    for i in 0..10 {
                        if i + 1 > num_posts {
                            stars
                                .get_with_index(u32::try_from(i).unwrap())
                                .unwrap()
                                .set_attribute("src", EMPTY_STAR_NAMES[i])
                                .unwrap();
                            continue;
                        }
                        stars
                            .get_with_index(u32::try_from(i).unwrap())
                            .unwrap()
                            .set_attribute("src", FILLED_STAR_NAMES[i])
                            .unwrap();
                    }
                });
            });

            let save_options: Callback<MouseEvent> = Callback::from(move |_| {
                let ok = get_document()
                    .get_element_by_id("uncensor")
                    .unwrap()
                    .unchecked_into::<web_sys::HtmlInputElement>();

                if ok.checked() {
                    LocalStorage::set("censor", true).unwrap();
                } else {
                    LocalStorage::delete("censor");
                }
            });

            let log_in: Callback<MouseEvent> = Callback::from(move |_| {
                let username = get_input("inputUsername");
                let password = get_input("inputPassword");

                if username.trim().is_empty() || password.trim().is_empty() {
                    set_text_str("a", "cannot be empty");
                    return;
                }

                spawn_local(async {
                    let resp = Request::post("/api/login")
                        .json(&User::new(username, password))
                        .unwrap()
                        .send()
                        .await;

                    match resp {
                        Ok(resp) => {
                            if resp.ok() {
                                if let Ok(Some(session)) = resp.json::<Option<String>>().await {
                                    LocalStorage::set("session", session).unwrap();
                                    set_text_str("a", "logged in!");
                                } else {
                                    set_text_str("a", "no session fetched");
                                }
                            } else {
                                match resp.status() {
                                    401 => set_text_str("a", "wrong password"),
                                    404 => set_text_str("a", "user not found"),
                                    500 => set_text_str("a", "internal server error"),
                                    _ => set_text_str("a", "unknown status"),
                                }
                            }
                        }
                        Err(err) => {
                            set_text("a", format!("request error: {err:?}"));
                        }
                    }
                });
            });

            let create_account: Callback<MouseEvent> = Callback::from(move |_| {
                let username = get_input("inputUsername");
                let password = get_input("inputPassword");

                if username.trim().is_empty() || password.trim().is_empty() {
                    set_text_str("a", "cannot be empty");
                    return;
                }

                spawn_local(async {
                    let resp = Request::post("/api/create_account")
                        .json(&User::new(username, password))
                        .unwrap()
                        .send()
                        .await;

                    match resp {
                        Ok(resp) => {
                            if resp.ok() {
                                if let Ok(Some(session)) = resp.json::<Option<String>>().await {
                                    LocalStorage::set("session", session).unwrap();
                                    set_text_str("a", "created!");
                                } else {
                                    set_text_str("a", "no session fetched");
                                }
                            } else {
                                match resp.status() {
                                    409 => set_text_str("a", "user already exists"),
                                    500 => set_text_str("a", "internal server error"),
                                    _ => set_text_str("a", "unknown status"),
                                }
                            }
                        }
                        Err(err) => {
                            set_text("a", format!("request error: {err:?}"));
                        }
                    }
                });
            });

            let logout: Callback<MouseEvent> = Callback::from(move |_| {
                if web_sys::window()
                    .unwrap()
                    .confirm_with_message("Are you sure you want to log out?")
                    .unwrap()
                {
                    LocalStorage::delete("session");
                    get_document().location().unwrap().reload().unwrap();
                }
            });

            let create_post: Callback<MouseEvent> = Callback::from(move |_| {
                let content: String = get_input("post_content");
                let session: String = format!("Bearer {}", match LocalStorage::get::<String>("session") {
                    Ok(session) => session,
                    Err(_) => {
                        set_text_str("b", "not logged in");
                        return;
                    },
                });

                if content.trim().is_empty() {
                    set_text_str("b", "cannot be empty");
                    return;
                }

                spawn_local(async move {
                    let resp = Request::post("/api/submit_post")
                        .header("authorization", &session)
                        .body(content)
                        .unwrap()
                        .send()
                        .await;

                    match resp {
                        Ok(resp) => {
                            if resp.ok() {
                                set_text("b", format!("ok! {}", resp.text().await.unwrap()));
                            } else {
                                set_text(
                                    "b",
                                    format!("server error: {}", resp.text().await.unwrap()),
                                );
                            }
                        }
                        Err(err) => set_text("b", format!("request error: {err}")),
                    }
                });
            });

            html! {
                <>

                <div class="row vw-100 vh-100 align-items-center">
                    <div class="col">
                        <h1 class="mb-3 text-center">{ "Post something" }</h1>

                        <div class="border rounded p-2">
                            <div class="mb-3">
                                <label for="inputContent" class="form-label">{ "What's on your mind?" }</label>
                                <input type="text" id="post_content" placeholder="Type here" class="form-control" aria-describedby="inputInfo" required=true/>

                                <div class="invalid-feedback">{"Please write something."}</div>

                                <div id="inputInfo" class="form-text">{ "Be mindful of what you post!" }</div>
                            </div>
                            <button onclick={create_post} class="btn btn-primary">{"Submit post"}</button>
                            <p id="b"/>
                        </div>
                    </div>

                    <div class="col">
                        <img src="/assets/tree.jpg" class="vh-100 mx-auto d-block" alt="A banyan tree."/>
                    </div>

                    <div class="col">
                        <h1 class="mb-3 text-center">{ "Manage account" }</h1>

                        <div class="d-flex flex-row justify-content-around align-items-center">
                            <h3 id="login-status"/>
                            <button class="btn btn-danger btn-sm" onclick={logout}>{"log out"}</button>
                        </div>

                        <div class="border rounded p-2 mb-2">
                            <div class="mb-3">
                                <label for="inputUsername" class="form-label">{ "Username" }</label>
                                <input type="text" placeholder="Type here" class="form-control" id="inputUsername" aria-describedby="usernameInfo" required=true/>

                                <div class="valid-feedback">{"Looks good!"}</div>
                                <div class="invalid-feedback">{"Please choose a username."}</div>

                                <div id="usernameInfo" class="form-text">{ "Usernames are unique!" }</div>
                            </div>
                            <div class="mb-3">
                                <label for="inputPassword" class="form-label">{ "Password" }</label>
                                <input class="form-control" id="inputPassword" placeholder="Type here" required=true/>

                                <div class="valid-feedback">{"Looks good!"}</div>
                                <div class="invalid-feedback">{"Please choose a password."}</div>
                            </div>

                            <button onclick={log_in} class="btn btn-primary">{ "Log in!" }</button>
                            <button onclick={create_account} class="btn btn-primary ms-3">{ "Create account!" }</button>
                            <p id="a"/>
                        </div>

                        <h1 class="mb-3 text-center">{ "Options" }</h1>

                        <div class="border rounded p-2">
                            <div class="mb-3 form-check">
                                <input type="checkbox" class="form-check-input" id="uncensor" name="uncensor"/>
                                <label for="uncensor" class="form-check-label">{ "Don't censor innapropriate words" }</label>
                            </div>

                            <button onclick={save_options} class="btn btn-primary">{ "Apply settings" }</button>
                        </div>
                    </div>
                </div>

                <a href="#">
                    <img onclick={get_new_posts} src="/assets/reload.png" style="position: absolute; top: -10%; right: 23%; transform: scale(0.28);" alt="Reload button."/>
                </a>

                <a href="javascript:show(\"post-1\");">
                    <img src="/assets/star-empty-1.png" class="star" style="position: absolute; top: 39.6%; left: 57%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"post-2\");">
                    <img src="/assets/star-empty-2.png" class="star" style="position: absolute; top: 42.7%; left: 50%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"post-3\");">
                    <img src="/assets/star-empty-3.png" class="star" style="position: absolute; top: 30%; left: 46%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"post-4\");">
                    <img src="/assets/star-empty-4.png" class="star" style="position: absolute; top: 14%; left: 37.9%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"post-5\");">
                    <img src="/assets/star-empty-5.png" class="star" style="position: absolute; top: 29.3%; left: 53.5%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"post-6\");">
                    <img src="/assets/star-empty-6.png" class="star" style="position: absolute; top: 15%; left: 45%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"post-7\");">
                    <img src="/assets/star-empty-7.png" class="star" style="position: absolute; top: 44.4%; left: 31.5%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"post-8\");">
                    <img src="/assets/star-empty-8.png" class="star" style="position: absolute; top: 36%; left: 34.3%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"post-9\");">
                    <img src="/assets/star-empty-9.png" class="star" style="position: absolute; top: 25%; left: 38%; transform: scale(0.55);" alt="A post in the form of a star."/>
                </a>

                <a href="javascript:show(\"post-10\");">
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

// async fn post_api_json<T, J>(path: &str, json: &J) -> Result<T, String>
// where
//     T: for<'a> Deserialize<'a>,
//     J: Serialize,
// {
//     let resp: Response = Request::post(path)
//         .json(json)
//         .unwrap()
//         .send()
//         .await
//         .unwrap();

//     let resp: Result<T, String> = if resp.ok() {
//         resp.json::<T>().await.map_err(|err| err.to_string())
//     } else {
//         Err(format!(
//             "Error fetching data {} ({}): \n{}",
//             resp.status(),
//             resp.status_text(),
//             resp.text().await.unwrap(),
//         ))
//     };

//     resp
// }

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

// async fn get_api_bearing(path: &str, auth: &str) -> Result<String, String> {
//     let resp: Response = Request::get(path)
//         .header("authorization", &format!("Bearer {auth}"))
//         .send()
//         .await
//         .unwrap();

//     let resp: Result<String, String> = if resp.ok() {
//         resp.text().await.map_err(|err| err.to_string())
//     } else {
//         Err(format!(
//             "Error fetching data {} ({})",
//             resp.status(),
//             resp.status_text()
//         ))
//     };

//     resp
// }

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
