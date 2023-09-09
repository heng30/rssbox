use crate::slint_generatedAppWindow::{RssConfig, RssList};

impl From<RssConfig> for RssList {
    fn from(conf: RssConfig) -> Self {
        RssList {
            name: conf.name,
            use_proxy: conf.use_proxy,
            icon_index: conf.icon_index,
            ..Default::default()
        }
    }
}
