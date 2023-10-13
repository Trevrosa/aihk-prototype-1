use gloo_net::http::{Request, Response};
use wasm_bindgen_futures::spawn_local;

use yew::prelude::*;
use yew_router::prelude::*;

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
        Route::Home => html! {
            <div>
                <h1>{ "Hello Frontend" }</h1>

                <br/>
                <h2>{ "One component" }</h2>
                <Hellos/>
                <br/>

                <h2>{ "Multiple components" }</h2>
                <HelloServer/>
                <HelloPython/>
                <HelloPYO3/>

            </div>
        },
        Route::NotFound => html! {
            <h1 style="text-align: center;">{ "404 Not Found" }</h1>
        },
    }
}

#[function_component(Hellos)]
fn hellos() -> Html {
    let data = use_state(|| None);

    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    let mut hellos = Vec::with_capacity(3);
                    hellos.push(get_api("/api/hello").await);
                    hellos.push(get_api("/api/python").await);
                    hellos.push(get_api("/api/pyo3").await);

                    data.set(Some(hellos));
                });
            }
        });
    }

    let hellos: Vec<String> = if let Some(data) = &*data {
        data.clone()
    } else {
        Vec::from([String::from("not found")])
    };

    html! {
        <>
        {
            hellos.iter().map(|hello| html!
                {
                    <p>{hello}</p>
                }
            ).collect::<Html>()
        }
        </>
    }
}

#[function_component(HelloPYO3)]
fn hello_pyo3() -> Html {
    let data = use_state(|| None);
    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    data.set(Some(get_api("/api/pyo3").await));
                });
            }
        });
    }

    html! {
        <p>{data.as_ref()}</p>
    }
}

#[function_component(HelloPython)]
fn hello_python() -> Html {
    let data = use_state(|| None);
    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    data.set(Some(get_api("/api/python").await));
                });
            }
        });
    }

    html! {
        <p>{data.as_ref()}</p>
    }
}

#[function_component(HelloServer)]
fn hello_server() -> Html {
    let data = use_state(|| None);

    // Request `/api/hello` once
    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    data.set(Some(get_api("/api/hello").await));
                });
            }
        });
    }

    html! {
        <p>{data.as_ref()}</p>
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

async fn get_api(path: &str) -> String {
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

    process_api_data(Some(&resp))
}

fn process_api_data(data: Option<&Result<String, String>>) -> String {
    match data {
        None => String::from("not found"),
        Some(Ok(data)) => data.clone(),
        Some(Err(err)) => err.clone(),
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
