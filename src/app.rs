use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>

        // sets the document title
        <Title text="Control Room"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AllowedCommand {
    Ls,
}

#[server]
pub async fn run_allowed_cmd(cmd: AllowedCommand) -> Result<String, ServerFnError>{
    match cmd {
        _Ls => Ok(
            String::from_utf8(
                Command::new("sh")
                    .arg("ls")
                    .output()
                    .expect("failed to execute")
                    .stdout
                ).expect("failed to parse output")
                .clone())
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (cmd_output, set_cmd_output) = create_signal(String::new());
    let on_click = move |_| {
                set_cmd_output.update(|output| *output = "yey".to_string())
        };

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Cmd ls"</button>
        <div>{move || cmd_output()}</div>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
