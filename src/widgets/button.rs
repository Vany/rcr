use crate::widgets::{Config, Error, Widget, WidgetRegistryEntry};
use tokio::sync::broadcast;
#[macros::register_widget]
pub struct Button<'a> {
    cfg: &'a Config,
    tx: &'a broadcast::Sender<String>,
}

impl<'a> Widget<'a> for Button<'a> {
    fn new(cfg: &'a Config, tx: &'a broadcast::Sender<String>) -> Box<dyn Widget<'a>> {
        Box::new(Button { cfg, tx })
    }

    fn dispatch(&mut self, _arg: &str) -> Result<(), Error> {
        Ok(())
    }
}
