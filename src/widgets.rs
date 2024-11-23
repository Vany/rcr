mod button;
mod container;

use tokio::sync::broadcast;

pub enum Error {}

pub struct Config {
    pub name: &'static str,
}

pub trait Widget<'a>: 'a {
    fn new(cfg: &'a Config, tx: &'a broadcast::Sender<String>) -> Box<dyn Widget<'a>>
    where
        Self: Sized;
    fn dispatch(&mut self, arg: &str) -> Result<(), Error>;
}

pub struct WidgetRegistryEntry<'a>(&'static str, WidgetConstructor<'a>);
pub type WidgetConstructor<'a> =
    fn(&'a Config, tx: &'a broadcast::Sender<String>) -> Box<dyn Widget<'a>>;
inventory::collect!(WidgetRegistryEntry<'static>);
#[macro_export]
macro_rules! register {
    ($i:ident) => {
        inventory::submit! { WidgetRegistryEntry(stringify!($i), $i::new) }
    };
}

pub fn print_all_widget_types() {
    let _cfg = Config { name: "print_all" };
    for WidgetRegistryEntry(name, _func) in inventory::iter::<WidgetRegistryEntry> {
        println!("{name}",);
    }
}
