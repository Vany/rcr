mod config;

use std::fmt::format;
use std::sync::{Arc};
use tokio::sync::Mutex;
use std::time::Duration;
use error_chain::error_chain;
use log::debug;
use simple_log;
use axum::{Router, routing::{get, any},
           extract::ws::{Message, WebSocket, WebSocketUpgrade, CloseFrame},  };
use axum::response::{Redirect, Response};
use tower_http::services::ServeDir;
use async_channel::{Receiver, Sender};
use axum::extract::State;

error_chain!(
    foreign_links {
        Cfg(::config::ConfigError);
        IO(std::io::Error);
    }
);

#[derive(Clone)]
struct AppState {
    wi_s_chan: Sender<String>,
    wi_r_chan: Receiver<String>,
}
impl AppState {
    fn new() -> Self {
        let (wi_s_chan, wi_r_chan) = async_channel::unbounded::<String>();
        Self{wi_s_chan, wi_r_chan}
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::Cfg::new()?;
    simple_log::quick!("debug");

    let listener = tokio::net::TcpListener::bind(&cfg.web.listen_addr).await.unwrap();
    println!("listening on {}", listener.local_addr()?);

    let app = Router::new()
        .route("/", get(Redirect::permanent(format!("/{}/index.html", cfg.web.static_dir).as_str())))
        .nest_service(format!("/{}", cfg.web.static_dir).as_str(), ServeDir::new(cfg.web.static_dir))
        .route("/ws", any(widgetws) )
        .with_state(AppState::new())
        ;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn widgetws(state: State<AppState>, ws: WebSocketUpgrade) -> Response {
    let (s, r) = (state.wi_s_chan.clone(), state.wi_r_chan.clone());
    ws.on_upgrade(move |ws| { handle_widgetws(ws, s, r) } )
}

async fn handle_widgetws(mut ws: WebSocket, s: Sender<String>, r: Receiver<String>) {
        if ws.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
            debug!("Ping out");
        } else {
            debug!("Could not ping out");
            return;
        }

    let wsm = Arc::new(Mutex::new(ws));

    // todo synchronise recver and sender closing
    let wsmc = wsm.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = r.recv().await {
            if let Err(err) = &wsmc.lock().await.send(Message::Text(msg)).await {
                debug!("Error sending message: {:?}", err);
                return
            }
        }
    });

    let wsmc = wsm.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = &wsmc.lock().await.recv().await {
            if let Message::Text(txt) = &msg {
                &s.send(txt.into()).await.unwrap();
            }
            debug!("web socket in : {msg:?}");
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => println!("{a:?} "),
                Err(a) => println!("Error sending messages {a:?}")
            }
            recv_task.abort();
        },
        rv_a = (&mut recv_task) => {
            match rv_a {
                Ok(b) => println!("Received {b:?} messages"),
                Err(b) => println!("Error receiving messages {b:?}")
            }
            send_task.abort();
        }
    }

    debug!("handle_widgetws exited");
}