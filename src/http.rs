use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::{Redirect, Response},
    routing::{any, get},
    Router,
};
use error_chain::error_chain;
use log::debug;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

error_chain!(
    foreign_links {
        IO(std::io::Error);
    }
);

#[derive(Clone)]
struct AppState {
    wi_s_chan: broadcast::Sender<String>,
}
impl AppState {
    fn new() -> Self {
        let (wi_s_chan, _wi_r_chan) = broadcast::channel(32);
        Self { wi_s_chan }
    }
}

pub async fn listen_and_serve(cfg: crate::config::Cfg) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(&cfg.web.listen_addr)
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr()?);

    let app = Router::new()
        .route(
            "/",
            get(Redirect::permanent(
                format!("/{}/index.html", cfg.web.static_dir).as_str(),
            )),
        )
        .nest_service(
            format!("/{}", cfg.web.static_dir).as_str(),
            ServeDir::new(cfg.web.static_dir),
        )
        .route("/ws", any(widgetws))
        .with_state(AppState::new());

    let server = axum::serve(listener, app);

    server.await?;
    Ok(())
}

async fn widgetws(state: State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |ws| {
        handle_widgetws(ws, state.wi_s_chan.clone(), state.wi_s_chan.subscribe())
    })
}

async fn handle_widgetws(
    ws: WebSocket,
    s: broadcast::Sender<String>,
    mut r: broadcast::Receiver<String>,
) {
    let mut ws = ws;
    let mut working = true;
    while working {
        tokio::select! {
            fromchan = r.recv() => {
                let fromchan = fromchan.unwrap();
                if let Err(_) = ws.send(Message::Text(fromchan)).await {
                    working = false;
                }
            },
            Some(Ok(msg)) = ws.recv() => {
                if let Message::Text(txt) = &msg {
                    s.send(txt.into()).unwrap();
                } else {
                    working = false;
                }
            },
        }
    }

    debug!("handle_widgetws exited");
}
