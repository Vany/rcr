mod config;
mod data;
mod http;
mod widgets;

use error_chain::error_chain;
use simple_log;

error_chain!(
    foreign_links {
        Cfg(::config::ConfigError);
        HTTP(http::Error);
    }
);

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::Cfg::new()?;
    simple_log::quick!("debug");

    widgets::print_all_widget_types();

    http::listen_and_serve(cfg).await?;
    Ok(())
}
