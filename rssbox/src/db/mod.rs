// pub mod archive;
pub mod data;
pub mod rss;

pub fn init() {
    rss::init().unwrap();
}
