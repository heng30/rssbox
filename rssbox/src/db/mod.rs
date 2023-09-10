pub mod data;
pub mod rss;
pub mod entry;

pub fn init() {
    rss::init().unwrap();
    entry::init().unwrap();
}
