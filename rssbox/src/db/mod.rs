pub mod data;
pub mod entry;
pub mod rss;

pub fn init() {
    rss::init().unwrap();
    entry::init().unwrap();
}
