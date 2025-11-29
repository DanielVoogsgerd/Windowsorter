use clap::Parser;
use std::sync::Arc;
use tracing::{debug, error, info};

use windowsorter::{decide_action, Action, Rules, Window};
mod config;
use config::ConfigFile;

/// Minimal CLI for windowsorter
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Don't actually move windows, just print the actions we would take
    #[arg(long, short)]
    dry_run: bool,
    /// Skip the initial scan of existing windows
    #[arg(long = "no-initial-scan")]
    no_initial_scan: bool,
}

// Top-level helper: process a single window (called from initial scan and event handler)
async fn process_window(
    class: String,
    workspace: u32,
    address: hyprland::shared::Address,
    title: Option<String>,
    dry: bool,
    rules: Arc<Rules>,
) {
    // Avoid printing window titles at info level (they can contain
    // personal information). Log class and workspace at info level and
    // titles at debug.
    info!("window: class={} ws={}", class, workspace);
    if let Some(ref t) = title {
        debug!("window details: title={}", t);
    }

    let w = Window { class: class.to_lowercase(), workspace };
    match decide_action(&w, rules.as_ref()) {
        Action::None => info!("no action for window"),
        Action::MoveTo(dest) => {
            if dry {
                info!("dry-run: would move window class={} to {}", class, dest);
                if let Some(t) = title { debug!("dry-run details: title={}", t); }
            } else {
                info!("moving window class={} to {}", class, dest);
                if let Some(t) = title { debug!("moving window details: title={}", t); }
                use hyprland::dispatch::{Dispatch, DispatchType, WindowIdentifier, WorkspaceIdentifierWithSpecial};
                let work = WorkspaceIdentifierWithSpecial::Id(dest as i32);
                let win = WindowIdentifier::Address(address);
                if let Err(e) = Dispatch::call_async(DispatchType::MoveToWorkspace(work, Some(win))).await {
                    error!("failed to dispatch move: {:?}", e);
                }
            }
        }
    }
}

/// Run initial scan: query currently existing clients/windows and apply rules.
async fn run_initial_scan(rules: Arc<Rules>, dry: bool) {
    use hyprland::data::Clients;
    // bring the HyprData trait into scope so get_async / get are available
    use hyprland::shared::HyprData;

    match Clients::get_async().await {
        Ok(clients) => {
            let mut seen = 0usize;
            for client in clients.into_iter() {
                seen += 1;
                // prefer parsing the workspace name (usually "1", "2", ...),
                // fall back to the numeric id if parsing fails.
                let ws_num = client
                    .workspace
                    .name
                    .parse::<i32>()
                    .map(|n| n as u32)
                    .unwrap_or(client.workspace.id as u32);

                let title = Some(client.initial_title.clone());
                process_window(client.initial_class.clone(), ws_num, client.address.clone(), title, dry, Arc::clone(&rules)).await;
            }
            info!("initial scan processed {} clients", seen);
        }
        Err(e) => info!("initial scan: could not query existing clients: {:?}. Continuing to listen for events.", e),
    }
}

/// Register event handlers and run the hyprland event listener.
async fn run_listener(rules: Arc<Rules>, dry: bool) {
    let mut listener = hyprland::event_listener::AsyncEventListener::new();

    let rules_clone = Arc::clone(&rules);
    listener.add_window_opened_handler(move |ev: hyprland::event_listener::WindowOpenEvent| {
        let rules = Arc::clone(&rules_clone);
        Box::pin(async move {
            // ev.workspace_name is often a numeric string like "1"
            let ws = ev.workspace_name.parse::<i32>().unwrap_or(0) as u32;
            process_window(ev.window_class, ws, ev.window_address, Some(ev.window_title), dry, rules).await;
        })
    });

    // Start the listener (blocks until socket closed)
    if let Err(e) = listener.start_listener_async().await {
        error!("event listener exited with error: {:?}", e);
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // init tracing: allow control via RUST_LOG (e.g. RUST_LOG=info,windowsorter=debug)
    use tracing_subscriber::EnvFilter;
    // Default to info-level logging; opt-in to debug/trace via RUST_LOG.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    // Try to load config file. If it's missing or empty, exit early — nothing to do.
    let cfg = match ConfigFile::load() {
        Ok(Some(c)) => Some(c),
        Ok(None) => None,
        Err(e) => {
            info!("failed to read config file: {:?}, falling back to defaults", e);
            None
        }
    };

    // If there is no config or the config contains no app entries, exit early.
    if cfg.as_ref().map_or(true, |c| c.app.is_empty()) {
        info!("no config or no app rules found; nothing to do, exiting");
        return;
    }

    let rules = Arc::new(cfg.unwrap().to_rules());

    info!("starting windowsorter (dry_run={})", cli.dry_run);

    // Only run initial scan when not explicitly disabled by CLI.
    if !cli.no_initial_scan {
        run_initial_scan(Arc::clone(&rules), cli.dry_run).await;
    } else {
        info!("initial scan disabled by CLI (--no-initial-scan), skipping")
    }

    run_listener(rules, cli.dry_run).await;
}
