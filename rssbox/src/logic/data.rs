use crate::slint_generatedAppWindow::{RssConfig, RssList};

impl From<RssConfig> for RssList {
    fn from(conf: RssConfig) -> Self {
        RssList {
            name: conf.name,
            url: conf.url,
            use_proxy: conf.use_proxy,
            icon_index: conf.icon_index,
            ..Default::default()
        }
    }
}

pub struct SyncItem {
    pub uuid: String,
    pub url: String,
}

impl From<RssList> for SyncItem {
    fn from(rss: RssList) -> Self {
        SyncItem {
            uuid: rss.uuid.to_string(),
            url: rss.url.to_string(),
        }
    }
}
