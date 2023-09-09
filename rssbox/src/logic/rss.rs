// use crate::db;
// use crate::db::data::SessionConfig;
// use crate::slint_generatedAppWindow::{AppWindow, Logic, Store};
use crate::slint_generatedAppWindow::{AppWindow};
// use crate::util::{self, translator::tr};
// #[allow(unused)]
// use log::{debug, warn};
// use slint::Timer;
// use slint::{ComponentHandle, Model, ModelExt, ModelRc, VecModel, Weak};
// use std::cmp::Ordering;
// use std::rc::Rc;
// use std::time::Duration;
// use uuid::Uuid;

// const DEFAULT_SESSION_UUID: &str = "default-session-uuid";

// fn init_db_default_session(ui: &AppWindow) {
//     for session in ui.global::<Store>().get_chat_sessions().iter() {
//         let uuid = session.uuid.to_string();

//         match db::session::is_exist(&uuid) {
//             Ok(exist) => {
//                 if exist {
//                     continue;
//                 }
//             }
//             Err(e) => warn!("{:?}", e),
//         }

//         let config_json = match serde_json::to_string(&SessionConfig::from(&session)) {
//             Ok(config) => config,
//             Err(e) => {
//                 ui.global::<Logic>().invoke_show_message(
//                     slint::format!("{}: {:?}", tr("设置默认会话库失败") + "!" + &tr("原因"), e),
//                     "warning".into(),
//                 );
//                 return;
//             }
//         };

//         if let Err(e) = db::session::insert(uuid, config_json, "".to_string()) {
//             ui.global::<Logic>().invoke_show_message(
//                 slint::format!(
//                     "{}: {:?}",
//                     tr("保存默认会话到数据库失败") + "!" + &tr("原因"),
//                     e
//                 ),
//                 "warning".into(),
//             );
//             return;
//         }
//     }
// }

// fn init_session(ui: &AppWindow) {
// match db::session::select_all() {
//     Ok(items) => {
//         let sessions = VecModel::default();

//         for item in items.into_iter() {
//             let config = item.1;
//             let screen_text = item.2.trim();

//             let mut chat_session = ChatSession {
//                 uuid: item.0.into(),
//                 ..Default::default()
//             };

//             match serde_json::from_str::<SessionConfig>(&config) {
//                 Ok(sc) => {
//                     chat_session.is_mark = sc.is_mark;
//                     chat_session.use_history = sc.use_history;
//                     chat_session.icon_index = sc.icon_index;
//                     chat_session.name = sc.name.into();
//                     chat_session.system_prompt = sc.system_prompt.into();
//                     chat_session.api_model = sc.api_model.into();
//                     chat_session.shortcut_instruction = sc.shortcut_instruction.into();
//                     chat_session.screen_text = screen_text.into();
//                 }
//                 Err(e) => {
//                     warn!("{:?}", e);
//                     continue;
//                 }
//             }

//             let chat_items = VecModel::default();

//             if !screen_text.is_empty() {
//                 chat_items.push(ChatItem {
//                     uuid: Uuid::new_v4().to_string().into(),
//                     timestamp: util::time::local_now("%Y-%m-%d %H:%M:%S").into(),
//                     utext: screen_text.into(),
//                     utext_items: chat::parse_chat_text(screen_text).into(),
//                     ..Default::default()
//                 });
//             }

//             chat_session.chat_items = Rc::new(chat_items).into();
//             chat_session.screen_items = chat_session.chat_items.clone();
//             sessions.push(chat_session);
//         }

//         let sessions = sessions.sort_by(|a, b| -> Ordering {
//             if a.uuid == DEFAULT_SESSION_UUID {
//                 Ordering::Less
//             } else if b.uuid == DEFAULT_SESSION_UUID {
//                 Ordering::Greater
//             } else if a.is_mark && b.is_mark {
//                 a.name.to_lowercase().cmp(&b.name.to_lowercase())
//             } else if a.is_mark && !b.is_mark {
//                 Ordering::Less
//             } else if !a.is_mark && b.is_mark {
//                 Ordering::Greater
//             } else {
//                 a.name.to_lowercase().cmp(&b.name.to_lowercase())
//             }
//         });

//         if sessions.row_count() > 0 {
//             ui.global::<Store>()
//                 .set_session_datas(sessions.row_data(0).unwrap().chat_items);
//             ui.global::<Logic>().invoke_show_session_archive_list(
//                 sessions.row_data(0).unwrap().uuid,
//                 ui.get_archive_search_text(),
//             );
//         }

//         ui.global::<Store>()
//             .set_chat_sessions(Rc::new(sessions).into());
//     }
//     Err(e) => {
//         warn!("{:?}", e);
//     }
// }
// }

pub fn init(ui: &AppWindow) {
    // init_db_default_session(ui);
    // init_session(ui);

    // let ui_handle = ui.as_weak();
    // ui.global::<Logic>().on_set_rss_dialog(move |string| {
    //     let ui = ui_handle.unwrap();
    //     // TODO
    // });

    // let ui_handle = ui.as_weak();
    // ui.global::<Logic>().on_reset_rss_dialog(move || {
    //     let ui = ui_handle.unwrap();
    //     // TODO
    // });

    // let ui_handle = ui.as_weak();
    // ui.global::<Logic>().on_new_session(move |config| {
    //     let ui = ui_handle.unwrap();
    //     let mut sessions: Vec<ChatSession> =
    //         ui.global::<Store>().get_chat_sessions().iter().collect();

    //     let cs = ChatSession {
    //         name: config.name,
    //         system_prompt: config.system_prompt,
    //         use_history: config.use_history,
    //         api_model: config.api_model,
    //         shortcut_instruction: config.shortcut_instruction,
    //         icon_index: config.icon_index,
    //         uuid: Uuid::new_v4().to_string().into(),
    //         ..Default::default()
    //     };

    //     let config_json = match serde_json::to_string(&db::data::SessionConfig::from(&cs)) {
    //         Ok(config) => config,
    //         Err(e) => {
    //             ui.global::<Logic>().invoke_show_message(
    //                 slint::format!("{}: {:?}", tr("保存到数据库失败") + "！" + &tr("原因"), e),
    //                 "warning".into(),
    //             );
    //             return;
    //         }
    //     };

    //     if let Err(e) = db::session::insert(cs.uuid.to_string(), config_json, "".to_string()) {
    //         ui.global::<Logic>().invoke_show_message(
    //             slint::format!("{}: {:?}", tr("保存到数据库失败") + "！" + &tr("原因"), e),
    //             "warning".into(),
    //         );
    //         return;
    //     }

    //     sessions.push(cs);
    //     let sessions_model = Rc::new(VecModel::from(sessions));
    //     ui.global::<Store>()
    //         .set_chat_sessions(sessions_model.into());
    //     ui.global::<Logic>()
    //         .invoke_show_message((tr("新建成功") + "！").into(), "success".into());
    // });

    // let ui_handle = ui.as_weak();
    // ui.global::<Logic>().on_delete_session(move |uuid| {
    //     let ui = ui_handle.unwrap();

    //     if uuid == DEFAULT_SESSION_UUID {
    //         ui.global::<Logic>()
    //             .invoke_show_message((tr("不允许删除默认会话") + "!").into(), "warning".into());
    //         return;
    //     }

    //     let sessions: Vec<ChatSession> = ui
    //         .global::<Store>()
    //         .get_chat_sessions()
    //         .iter()
    //         .filter(|x| x.uuid != uuid)
    //         .collect();

    //     if let Err(e) = db::session::delete(uuid.to_string()) {
    //         ui.global::<Logic>().invoke_show_message(
    //             slint::format!("{}: {:?}", tr("删除会话失败") + "!" + &tr("原因"), e),
    //             "warning".into(),
    //         );
    //         return;
    //     }

    //     ui.global::<Store>()
    //         .set_current_session_uuid(DEFAULT_SESSION_UUID.into());

    //     let sessions_model = Rc::new(VecModel::from(sessions));
    //     if sessions_model.row_count() > 0 {
    //         ui.global::<Store>()
    //             .set_session_datas(sessions_model.row_data(0).unwrap().chat_items);
    //     }

    //     ui.global::<Store>()
    //         .set_chat_sessions(sessions_model.into());
    //     ui.global::<Logic>()
    //         .invoke_show_message((tr("删除会话成功") + "!").into(), "success".into());
    // });

    // let ui_handle = ui.as_weak();
    // ui.global::<Logic>().on_toggle_rss_mark(move |uuid| {
    //     let ui = ui_handle.unwrap();
    //     let sessions: Vec<ChatSession> = ui
    //         .global::<Store>()
    //         .get_chat_sessions()
    //         .iter()
    //         .map(|x| {
    //             if x.uuid != uuid {
    //                 x
    //             } else {
    //                 let mut m = x.clone();
    //                 m.is_mark = !x.is_mark;
    //                 m
    //             }
    //         })
    //         .collect();

    //     let mut is_mark = false;
    //     for cs in sessions.iter() {
    //         if cs.uuid != uuid {
    //             continue;
    //         }

    //         is_mark = cs.is_mark;

    //         match serde_json::to_string(&SessionConfig::from(cs)) {
    //             Ok(config) => {
    //                 if let Err(e) = db::session::update(uuid.to_string(), Some(config), None) {
    //                     ui.global::<Logic>().invoke_show_message(
    //                         slint::format!(
    //                             "{}: {:?}",
    //                             tr("保存到数据库失败") + "！" + &tr("原因"),
    //                             e
    //                         ),
    //                         "warning".into(),
    //                     );
    //                     return;
    //                 }
    //                 break;
    //             }
    //             Err(e) => {
    //                 ui.global::<Logic>().invoke_show_message(
    //                     slint::format!("{}: {:?}", tr("保存到数据库失败") + "！" + &tr("原因"), e),
    //                     "warning".into(),
    //                 );
    //                 return;
    //             }
    //         };
    //     }

    //     let sessions_model = Rc::new(VecModel::from(sessions));
    //     ui.global::<Store>()
    //         .set_chat_sessions(sessions_model.into());

    //     if is_mark {
    //         ui.global::<Logic>()
    //             .invoke_show_message((tr("收藏成功") + "！").into(), "success".into());
    //     } else {
    //         ui.global::<Logic>()
    //             .invoke_show_message((tr("取消收藏成功") + "！").into(), "success".into());
    //     }
    // });

    // let ui_handle = ui.as_weak();
    // ui.global::<Logic>()
    //     .on_save_edit_session(move |uuid, config| {
    //         let ui = ui_handle.unwrap();
    //         let sessions: Vec<ChatSession> = ui
    //             .global::<Store>()
    //             .get_chat_sessions()
    //             .iter()
    //             .map(|x| {
    //                 if x.uuid != uuid {
    //                     x
    //                 } else {
    //                     ChatSession {
    //                         name: config.name.clone(),
    //                         system_prompt: config.system_prompt.clone(),
    //                         api_model: config.api_model.clone(),
    //                         shortcut_instruction: config.shortcut_instruction.clone(),
    //                         icon_index: config.icon_index,
    //                         use_history: config.use_history,
    //                         ..x
    //                     }
    //                 }
    //             })
    //             .collect();

    //         for session in sessions.iter() {
    //             if session.uuid == uuid {
    //                 match serde_json::to_string(&SessionConfig::from(session)) {
    //                     Ok(config) => {
    //                         if let Err(e) =
    //                             db::session::update(uuid.to_string(), Some(config), None)
    //                         {
    //                             ui.global::<Logic>().invoke_show_message(
    //                                 slint::format!(
    //                                     "{}: {:?}",
    //                                     tr("保存会话失败") + "！" + &tr("原因"),
    //                                     e
    //                                 ),
    //                                 "warning".into(),
    //                             );
    //                             return;
    //                         }
    //                         break;
    //                     }
    //                     Err(e) => {
    //                         ui.global::<Logic>().invoke_show_message(
    //                             slint::format!(
    //                                 "{}: {:?}",
    //                                 tr("保存会话配置失败") + "！" + &tr("原因"),
    //                                 e
    //                             ),
    //                             "warning".into(),
    //                         );
    //                         return;
    //                     }
    //                 };
    //             }
    //         }

    //         let sessions_model = Rc::new(VecModel::from(sessions));
    //         ui.global::<Store>()
    //             .set_chat_sessions(sessions_model.into());
    //         ui.global::<Logic>()
    //             .invoke_show_message((tr("保存会话配置成功") + "!").into(), "success".into());
    //     });

    // let ui_handle = ui.as_weak();
    // ui.global::<Logic>()
    //     .on_switch_session(move |old_uuid, new_uuid| {
    //         if old_uuid == new_uuid || new_uuid.is_empty() {
    //             return;
    //         }

    //         let ui = ui_handle.unwrap();
    //         let chat_items = ui.global::<Store>().get_session_datas();
    //         let chats_viewport_y = ui.get_chats_viewport_y();
    //         let sessions = ui.global::<Store>().get_chat_sessions();

    //         let mut index = 0;
    //         for (row, session) in sessions.iter().enumerate() {
    //             if session.uuid == old_uuid {
    //                 ui.global::<Store>().get_chat_sessions().set_row_data(
    //                     row,
    //                     ChatSession {
    //                         chats_viewport_y,
    //                         chat_items: chat_items.clone(),
    //                         ..session
    //                     },
    //                 );

    //                 index += 1;
    //             } else if session.uuid == new_uuid {
    //                 ui.global::<Store>()
    //                     .set_session_datas(session.chat_items.clone());

    //                 // join the cache text that recieved in background
    //                 let row_count = ui.global::<Store>().get_session_datas().row_count();

    //                 ui.set_archive_search_text("".into());
    //                 ui.global::<Logic>().invoke_show_session_archive_list(
    //                     new_uuid.clone(),
    //                     ui.get_archive_search_text(),
    //                 );
    //                 ui.global::<Store>()
    //                     .set_previous_session_uuid(old_uuid.clone());
    //                 ui.global::<Store>()
    //                     .set_current_session_uuid(new_uuid.clone());
    //                 ui.invoke_archive_scroll_to_top();
    //                 ui.set_chats_viewport_y(session.chats_viewport_y);

    //                 let new_y = session.chats_viewport_y;
    //                 for i in 1..=3 {
    //                     let ui = ui.as_weak();
    //                     Timer::single_shot(Duration::from_millis(i * 100), move || {
    //                         let ui = ui.unwrap();
    //                         if ui.get_chats_viewport_y() == new_y {
    //                             return;
    //                         }

    //                         ui.set_chats_viewport_y(new_y);
    //                     });
    //                 }

    //                 index += 1;
    //             }

    //             if index == 2 {
    //                 break;
    //             }
    //         }
    //     });
}
