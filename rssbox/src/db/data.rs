// use crate::slint_generatedAppWindow::{RssConfig as UIRssConfig};
// use std::fmt::Debug;

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct RssConfig {
//     pub name: String,
//     pub icon_index: i32,
//     pub use_proxy: bool,
//     pub is_mark: bool,
//     pub update_time: String,
// }

// impl From<&UIRssConfig> for RssConfig {
//     fn from(conf: &UIRssConfig) -> Self {
//         RssConfig {
//             name: conf.name.clone().into(),
//             icon_index: conf.icon_index,
//             use_proxy: conf.use_proxy,
//             is_mark: conf.is_mark,
//             update_time: conf.update_time.clone().into(),
//         }
//     }
// }
