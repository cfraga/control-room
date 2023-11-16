use leptos::{*, html::output};
use leptos_meta::*;
use leptos_router::*;
extern crate chrono;
use chrono::{Utc, DateTime};
use std::{process::Command, fmt, ops};
use serde::{Deserialize, Serialize};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/assets/test.css"/>

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
    output: Option<String>,
    status: Option<bool>,
    timestamp: DateTime<Utc>,
}

impl RemoteOp {
    fn from_op(op: AllowedOperation) -> RemoteOp {
        RemoteOp { op: op, output: None, status: None, timestamp: Utc::now()}
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
    TmuxList,
    Shutdown,
}

impl std::fmt::Display for AllowedCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ls => write!(f, "ls"),
            Self::Pwd => write!(f, "pwd"),
            Self::Shutdown => write!(f, "shutdown"),
            Self::TmuxList => write!(f, "tmux list")
        }
    }
}


impl AllowedCommand {
    fn value(&self) -> &str {
        match *self {
            AllowedCommand::Ls => "ls",
            AllowedCommand::Pwd => "pwd",
            AllowedCommand::Shutdown => "shutdown",
            AllowedCommand::TmuxList => "tmux",
        }
    }

    fn args(&self) -> Option<Vec<&str>>{
        match *self {
            AllowedCommand::Ls => None,
            AllowedCommand::Pwd => None,
            AllowedCommand::Shutdown => Some(vec!["-h", "now"]),
            AllowedCommand::TmuxList => Some(vec!["list-sessions"]),
        }
    }
}
#[server(RunAllowedOp)]
pub async fn run_allowed_op(cmd: AllowedOperation) -> Result<RemoteOp, ServerFnError>{
    let output = match cmd {
        AllowedOperation::ShellCommand(cmd) => run_cmd(cmd).await,
        AllowedOperation::ShellScript => Err(ServerFnError::ServerError("Not Supported yet".to_string())),
        AllowedOperation::Noop => Ok(RemoteOp {op: cmd, output: Some("Nothing was done".to_string()), status: Some(true), timestamp: Utc::now()})
    };
    logging::log!("{:#?}",output);

    output
}

#[server]
pub async fn run_cmd(cmd: AllowedCommand) -> Result<RemoteOp, ServerFnError>{
    let mut cmd_exec = Command::new(cmd.value());
    for arg in cmd.args().iter().flatten() {
        cmd_exec.arg(arg);
    }

    match cmd_exec.output() {
        Ok(output) => {
            let mut stdio = String::from_utf8(output.stdout).expect("failed to parse output");
            stdio.push_str("\n");
            stdio.push_str(&String::from_utf8(output.stderr).expect("failed to parse output"));
            Ok(RemoteOp {op: AllowedOperation::ShellCommand(cmd), output: Some(stdio), status: Some(output.status.success()), timestamp: Utc::now()})
        },
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

#[component]
fn OperationButton(exec_op: AllowedOperation, run_action: Action<AllowedOperation, Result<RemoteOp, ServerFnError>>, label: Option<String>) -> impl IntoView {
    let on_click = move |_| {
        let cloned_op = exec_op.clone();
        logging::log!("attempting {}...", cloned_op.to_string());
        if !run_action.pending().get() { 
            run_action.dispatch(cloned_op);
        }
    };

    view! {
        <div class="op-command-enabled" on:click=on_click disabled={move || run_action.pending().get()   }>{label}</div>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (ops_log, set_ops_log) = create_signal(Vec::<RemoteOp>::new());
    
    let run_op = create_action(
        |op: &AllowedOperation| {
            let op = op.clone();
            logging::log!("executing op");
            async move { run_allowed_op(op).await }
        });

    create_effect(move |_| {
        if let Some(Ok(remote_op)) = run_op.value().get() {
            logging::log!("updating ops log");
            set_ops_log.update(|log| log.push(remote_op));
            logging::log!("ops_log_count: {}", ops_log().len());
        }
    });

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <OperationButton exec_op=AllowedOperation::ShellCommand(AllowedCommand::Ls) run_action=run_op label=Some("LS".to_string()) />
        <OperationButton exec_op=AllowedOperation::ShellCommand(AllowedCommand::Pwd) run_action=run_op label=Some("PWD".to_string()) />
        <OperationButton exec_op=AllowedOperation::ShellCommand(AllowedCommand::TmuxList) run_action=run_op label=Some("Tmux Sessions".to_string()) />
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            <div>

                <For each=ops_log
                    key=|op| { op.timestamp.to_string() }
                    let:child>
                    <div>
                        <div>
                            <span>{child.timestamp.format("%Y-%m-%d %H:%M:%S > ").to_string()}</span><span>{child.output}</span>
                        </div>
                    </div>
            </For>
            </div>
        </Suspense>
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
