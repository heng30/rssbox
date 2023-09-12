use crate::config;
use crate::db;
use crate::db::data::RssEntry;
use crate::rss;
use crate::slint_generatedAppWindow::{AppWindow, Logic, RssEntry as UIRssEntry, Store};
use crate::util::translator::tr;
use cmd_lib::run_cmd;
use log::warn;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use tokio::task::spawn;
use webbrowser;

pub fn get_from_db(suuid: &str) -> Vec<UIRssEntry> {
    if suuid == rss::UNREAD_UUID {
        return vec![];
    }

    let mut entrys = vec![];
    match db::entry::select_all(suuid) {
        Ok(items) => {
            entrys = items
                .iter()
                .rev()
                .map(|item| match serde_json::from_str::<RssEntry>(&item.1) {
                    Ok(entry) => entry.into(),
                    Err(e) => {
                        warn!("{:?}", e);
                        UIRssEntry::default()
                    }
                })
                .collect();
            entrys.sort_by(|a, b| a.is_read.cmp(&b.is_read));
        }
        Err(e) => {
            warn!("{:?}", e);
        }
    }
    entrys
}

pub fn update_new_entry(ui: &AppWindow, suuid: &str, entry: RssEntry) {
    match serde_json::to_string(&entry) {
        Ok(data) => {
            if suuid != rss::UNREAD_UUID {
                if let Err(e) = db::entry::insert(suuid, entry.uuid.as_str(), &data) {
                    warn!("{:?}", e);
                    return;
                }
            }

            for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
                if rss.uuid.as_str() != suuid {
                    continue;
                }
                rss.entry
                    .as_any()
                    .downcast_ref::<VecModel<UIRssEntry>>()
                    .expect("We know we set a VecModel earlier")
                    .insert(0, entry.into());

                rss.unread_count = rss.unread_count + 1;
                ui.global::<Store>()
                    .get_rss_lists()
                    .set_row_data(index, rss);
                break;
            }
        }
        Err(e) => {
            warn!("{:?}", e);
        }
    };
}

pub fn init(ui: &AppWindow) {
    ui.global::<Logic>().on_parse_tags(move |tags| {
        let items: Vec<_> = tags
            .split(',')
            .map(|tag| SharedString::from(tag.trim()))
            .collect();

        ModelRc::new(VecModel::from(items))
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_open_url(move |url| {
        let ui = ui_handle.unwrap();
        let rss_config = config::rss();

        if rss_config.browser.is_empty() {
            if let Err(e) = webbrowser::open(url.as_str()) {
                ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}{}: {:?}", tr("打开链接失败！"), tr("原因"), e),
                    "warning".into(),
                );
            }
        } else {
            let browser = rss_config.browser.clone();
            let url = url.to_string();
            let ui_handle = ui.as_weak();

            spawn(async move {
                if let Err(e) = run_cmd!($browser $url) {
                    let err = format!("{:?}", e);
                    if let Err(e) = slint::invoke_from_event_loop(move || {
                        ui_handle.unwrap().global::<Logic>().invoke_show_message(
                            slint::format!("{}{}: {:?}", tr("打开链接失败！"), tr("原因"), err),
                            "warning".into(),
                        );
                    }) {
                        warn!("{:?}", e);
                    }
                }
            });
        }
    });

    let set_read = |ui: &AppWindow,
                    suuid: &slint::SharedString,
                    index: usize,
                    mut entry: UIRssEntry| {
        entry.is_read = true;

        match serde_json::to_string(&RssEntry::from(&entry)) {
            Ok(data) => {
                if suuid.as_str() != rss::UNREAD_UUID {
                    if let Err(e) = db::entry::update(suuid.as_str(), entry.uuid.as_str(), &data) {
                        ui.global::<Logic>().invoke_show_message(
                            slint::format!("{}{}: {:?}", tr("保存失败！"), tr("原因"), e),
                            "warning".into(),
                        );
                    }
                }
            }
            Err(e) => {
                ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}{}: {:?}", tr("保存失败！"), tr("原因"), e),
                    "warning".into(),
                );
            }
        };

        ui.global::<Store>()
            .get_rss_entry()
            .set_row_data(index, entry);
    };

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_set_read_all_entry(move |suuid| {
        let ui = ui_handle.unwrap();

        for (index, entry) in ui.global::<Store>().get_rss_entry().iter().enumerate() {
            if entry.is_read {
                continue;
            }
            set_read(&ui, &suuid, index, entry);
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_set_read_entry(move |suuid, uuid| {
        let ui = ui_handle.unwrap();

        for (index, entry) in ui.global::<Store>().get_rss_entry().iter().enumerate() {
            if entry.is_read || entry.uuid != uuid {
                continue;
            }

            set_read(&ui, &suuid, index, entry);
            return;
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_all_entry(move |suuid| {
        let ui = ui_handle.unwrap();

        ui.global::<Store>()
            .get_rss_entry()
            .as_any()
            .downcast_ref::<VecModel<UIRssEntry>>()
            .expect("We know we set a VecModel earlier")
            .set_vec(vec![]);

        if suuid != rss::UNREAD_UUID {
            if let Err(e) = db::entry::delete_all(suuid.as_str()) {
                ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}{}: {:?}", tr("清空失败！"), tr("原因"), e),
                    "warning".into(),
                );
            } else {
                ui.global::<Logic>()
                    .invoke_show_message(tr("清空成功！").into(), "success".into());
            }
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_entry(move |suuid, uuid| {
        let ui = ui_handle.unwrap();

        for (index, entry) in ui.global::<Store>().get_rss_entry().iter().enumerate() {
            if entry.uuid != uuid {
                continue;
            }

            if suuid != rss::UNREAD_UUID {
                if let Err(e) = db::entry::delete(suuid.as_str(), entry.uuid.as_str()) {
                    ui.global::<Logic>().invoke_show_message(
                        slint::format!("{}{}: {:?}", tr("删除失败！"), tr("原因"), e),
                        "warning".into(),
                    );
                } else {
                    ui.global::<Logic>()
                        .invoke_show_message(tr("删除成功！").into(), "success".into());
                }
            }

            ui.global::<Store>()
                .get_rss_entry()
                .as_any()
                .downcast_ref::<VecModel<UIRssEntry>>()
                .expect("We know we set a VecModel earlier")
                .remove(index);

            return;
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_favorite_entry(move |_suuid, uuid| {
        let ui = ui_handle.unwrap();

        for mut entry in ui.global::<Store>().get_rss_entry().iter() {
            if entry.uuid != uuid {
                continue;
            }

            // TODO: set tag
            // entry.tags = rss
            entry.is_read = true;

            match serde_json::to_string(&RssEntry::from(&entry)) {
                Ok(data) => {
                    for rss in ui.global::<Store>().get_rss_lists().iter() {
                        if rss.uuid != rss::FAVORITE_UUID {
                            continue;
                        }

                        let mut found = false;
                        for item in rss.entry.iter() {
                            if item.url == entry.url {
                                found = true;
                                break;
                            }
                        }

                        if found {
                            ui.global::<Logic>()
                                .invoke_show_message(tr("已经收藏！").into(), "info".into());
                        } else {
                            if let Err(e) =
                                db::entry::insert(rss::FAVORITE_UUID, entry.uuid.as_str(), &data)
                            {
                                ui.global::<Logic>().invoke_show_message(
                                    slint::format!("{}{}: {:?}", tr("保存失败！"), tr("原因"), e),
                                    "warning".into(),
                                );
                            }

                            rss.entry
                                .as_any()
                                .downcast_ref::<VecModel<UIRssEntry>>()
                                .expect("We know we set a VecModel earlier")
                                .insert(0, entry);

                            ui.global::<Logic>()
                                .invoke_show_message(tr("收藏成功！").into(), "success".into());
                        }

                        break;
                    }
                }
                Err(e) => {
                    ui.global::<Logic>().invoke_show_message(
                        slint::format!("{}{}: {:?}", tr("保存失败！"), tr("原因"), e),
                        "warning".into(),
                    );
                }
            };

            return;
        }
    });
}
