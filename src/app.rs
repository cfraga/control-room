use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use std::{process::Command, fmt};
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RemoteOp {
    op: AllowedOperation,
    last_output: Option<String>,
    last_status: Option<bool>
}

impl Default for RemoteOp {
    fn default() -> Self {
        Self {
            op: AllowedOperation::Noop,
            last_output: None,
            last_status: None,
        }
    }
}

impl RemoteOp {
    fn is_running(&self) -> bool{
        self.op != AllowedOperation::Noop && self.last_status.is_none()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AllowedOperation {
    Noop,
    ShellCommand(AllowedCommand),
    ShellScript,
}

impl std::fmt::Display for AllowedOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Noop => write!(f, "Reset"),
            Self::ShellCommand(cmd) => write!(f, "Cmd: {}", cmd),
            Self::ShellScript => write!(f, "Script: "),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AllowedCommand {
    Ls,
    Pwd,
    Shutdown,
}

impl std::fmt::Display for AllowedCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ls => write!(f, "ls"),
            Self::Pwd => write!(f, "pwd"),
            Self::Shutdown => write!(f, "shutdown"),
        }
    }
}


impl AllowedCommand {
    fn value(&self) -> &str {
        match *self {
            AllowedCommand::Ls => "ls",
            AllowedCommand::Pwd => "pwd",
            AllowedCommand::Shutdown => "shutdown"
        }
    }

    fn args(&self) -> Option<Vec<&str>>{
        match *self {
            AllowedCommand::Ls => None,
            AllowedCommand::Pwd => None,
            AllowedCommand::Shutdown => Some(vec!["-h", "now"]),
        }
    }
}
#[server]
pub async fn run_allowed_op(cmd: AllowedOperation) -> Result<String, ServerFnError>{
    let output = match cmd {
        AllowedOperation::ShellCommand(cmd) => run_cmd(cmd).await,
        AllowedOperation::ShellScript => Ok("Not Supported yet".to_string()),
        AllowedOperation::Noop => Ok("NO OP".to_string())
    };
    logging::log!("{:#?}",output);
    output
}

#[server]
pub async fn run_cmd(cmd: AllowedCommand) -> Result<String, ServerFnError>{
    Ok(String::from_utf8(
            Command::new(cmd.value())
            // .args(&cmd.args().ok_or([]))
            .output()
            .expect("failed to execute")
            .stdout)
        .expect("failed to parse output")
        .clone())
}

#[component]
fn OperationButton(exec_op: AllowedOperation, current_op: ReadSignal<RemoteOp>, set_current_op: WriteSignal<RemoteOp>, label: Option<String>) -> impl IntoView {
    let cloned_op = exec_op.clone();
    let on_click = move |_| {
        logging::log!("clicking button. is running? {}", current_op().is_running());
        if !current_op().is_running() {
            set_current_op.update(|remote_op| *remote_op = RemoteOp { op: cloned_op.clone(), last_status: None, last_output: None })
        }
    };

    view! {
        <button on:click=on_click disabled={move || current_op().is_running()}>"random text"</button>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (current_op, set_current_op) = create_signal(RemoteOp::default());

    let op_exec = create_resource(
        current_op, 
        |op| async move {
            logging::log!("executing op");
            run_allowed_op(op.op).await 
        });
    
 
    view! {
        <h1>"Welcome to Leptos!"</h1>
        <OperationButton exec_op=AllowedOperation::ShellCommand(AllowedCommand::Ls) current_op=current_op set_current_op=set_current_op label=Some("LS".to_string()) />
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            <div>
            {move || match op_exec() {
                None => "Click a button plzz?".to_string(),
                Some(output) => output.unwrap()
            }}
            </div>
            </Suspense>
    }
}

// #[component]
// fn CommandButton(cmd: AllowedCommand) -> impl IntoView {

// }

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
