pub mod data;
pub mod entry;
pub mod rss;
pub mod trash;

pub fn init() {
    rss::init().unwrap();
    entry::init().unwrap();
    trash::init().unwrap();
}
