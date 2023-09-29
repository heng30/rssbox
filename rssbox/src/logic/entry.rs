use crate::config;
use crate::db;
use crate::db::data::RssEntry;
use crate::rss;
use crate::slint_generatedAppWindow::{
    AppWindow, Logic, RssEntry as UIRssEntry, RssList as UIRssList, Store,
};
use crate::util::{crypto::md5_hex, translator::tr};
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
                    Ok(entry) => {
                        let mut entry: UIRssEntry = entry.into();
                        entry.suuid = suuid.into();
                        entry
                    }
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

pub fn update_new_entry(ui: &AppWindow, real_suuid: &str, suuid: &str, entry: RssEntry) {
    match serde_json::to_string(&entry) {
        Ok(data) => {
            if suuid != rss::UNREAD_UUID {
                if let Err(e) = db::entry::insert(suuid, entry.uuid.as_str(), &data) {
                    warn!("{:?}", e);
                    return;
                }
            }

            for (index, rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
                if rss.uuid.as_str() != suuid {
                    continue;
                }
                rss.entry
                    .as_any()
                    .downcast_ref::<VecModel<UIRssEntry>>()
                    .expect("We know we set a VecModel earlier")
                    .insert(0, {
                        let mut entry: UIRssEntry = entry.into();
                        entry.suuid = real_suuid.into();
                        entry
                    });

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

fn get_rsslist(ui: &AppWindow, suuid: &str) -> UIRssList {
    for rss in ui.global::<Store>().get_rss_lists().iter() {
        if rss.uuid.as_str() == suuid {
            return rss;
        }
    }
    return UIRssList::default();
}

fn set_read_all_entry(ui: &AppWindow, suuid: &str) {
    for entry in ui.global::<Store>().get_rss_entry().iter() {
        if entry.is_read {
            continue;
        }
        set_read_entry(&ui, suuid, entry.uuid.as_str());
    }
}

fn set_read_entry(ui: &AppWindow, suuid: &str, uuid: &str) {
    for rss in ui.global::<Store>().get_rss_lists().iter() {
        if rss.uuid != suuid {
            continue;
        }

        for (index, mut entry) in rss.entry.iter().enumerate() {
            if entry.uuid == uuid {
                if entry.is_read {
                    return;
                }

                entry.is_read = true;

                match serde_json::to_string(&RssEntry::from(&entry)) {
                    Ok(data) => {
                        if suuid != rss::UNREAD_UUID {
                            if let Err(e) = db::entry::update(suuid, entry.uuid.as_str(), &data) {
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

                rss.entry
                    .as_any()
                    .downcast_ref::<VecModel<UIRssEntry>>()
                    .expect("We know we set a VecModel earlier")
                    .set_row_data(index, entry);

                return;
            }
        }

        return;
    }
}

fn remove_all_entry(ui: &AppWindow, suuid: &str) {
    for entry in ui.global::<Store>().get_rss_entry().iter() {
        let _ = db::trash::insert(&md5_hex(entry.url.as_str()));
    }

    ui.global::<Store>()
        .get_rss_entry()
        .as_any()
        .downcast_ref::<VecModel<UIRssEntry>>()
        .expect("We know we set a VecModel earlier")
        .set_vec(vec![]);

    if suuid != rss::UNREAD_UUID {
        if let Err(e) = db::entry::delete_all(suuid) {
            ui.global::<Logic>().invoke_show_message(
                slint::format!("{}{}: {:?}", tr("清空失败！"), tr("原因"), e),
                "warning".into(),
            );
        } else {
            ui.global::<Logic>()
                .invoke_show_message(tr("清空成功！").into(), "success".into());
        }
    }
}

fn remove_entry(ui: &AppWindow, suuid: &str, uuid: &str) {
    for rss in ui.global::<Store>().get_rss_lists().iter() {
        if rss.uuid != suuid {
            continue;
        }

        for (index, entry) in rss.entry.iter().enumerate() {
            if entry.uuid != uuid {
                continue;
            }

            if suuid != rss::UNREAD_UUID {
                if let Err(e) = db::entry::delete(suuid, uuid) {
                    ui.global::<Logic>().invoke_show_message(
                        slint::format!("{}{}: {:?}", tr("删除失败！"), tr("原因"), e),
                        "warning".into(),
                    );
                } else {
                    ui.global::<Logic>()
                        .invoke_show_message(tr("删除成功！").into(), "success".into());
                }

                let _ = db::trash::insert(&md5_hex(entry.url.as_str()));
            }

            rss.entry
                .as_any()
                .downcast_ref::<VecModel<UIRssEntry>>()
                .expect("We know we set a VecModel earlier")
                .remove(index);

            return;
        }
        return;
    }
}

pub fn init(ui: &AppWindow) {
    ui.global::<Logic>().on_parse_tags(move |tags| {
        let items: Vec<_> = tags
            .trim()
            .trim_start_matches(',')
            .trim_end_matches(',')
            .split(',')
            .map(|tag| SharedString::from(tag.trim()))
            .filter(|tag| !tag.is_empty())
            .collect();

        ModelRc::new(VecModel::from(items))
    });

    ui.global::<Logic>()
        .on_unread_counts(move |entrys, _counts, _flag| {
            let mut counts = 0;
            for entry in entrys.iter() {
                if !entry.is_read {
                    counts += 1;
                }
            }
            counts
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

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_set_read_all_entry(move |suuid| {
        if suuid == rss::FAVORITE_UUID {
            return;
        }

        let ui = ui_handle.unwrap();

        for rss in ui.global::<Store>().get_rss_lists().iter() {
            if rss.uuid != suuid {
                continue;
            }

            for entry in rss.entry.iter() {
                if entry.is_read {
                    continue;
                }

                set_read_entry(
                    &ui,
                    if suuid == rss::UNREAD_UUID {
                        entry.suuid.as_str()
                    } else {
                        rss::UNREAD_UUID
                    },
                    entry.uuid.as_str(),
                );
            }

            break;
        }
        set_read_all_entry(&ui, suuid.as_str());

        ui.global::<Store>().invoke_toggle_unread_count_flag();
        if suuid == rss::UNREAD_UUID {
            for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
                rss.unread_counts_flag = !rss.unread_counts_flag;
                ui.global::<Store>()
                    .get_rss_lists()
                    .set_row_data(index, rss);
            }
        } else {
            for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
                if rss.uuid == rss::UNREAD_UUID || rss.uuid == suuid {
                    rss.unread_counts_flag = !rss.unread_counts_flag;
                    let is_break = rss.uuid == suuid;
                    ui.global::<Store>()
                        .get_rss_lists()
                        .set_row_data(index, rss);

                    if is_break {
                        break;
                    }
                }
            }
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_set_read_entry(move |suuid, uuid| {
        if suuid == rss::FAVORITE_UUID {
            return;
        }

        let ui = ui_handle.unwrap();
        if suuid == rss::UNREAD_UUID {
            for rss in ui.global::<Store>().get_rss_lists().iter() {
                if rss.uuid != suuid {
                    continue;
                }

                for entry in rss.entry.iter() {
                    if entry.uuid == uuid {
                        set_read_entry(&ui, entry.suuid.as_str(), uuid.as_str());

                        break;
                    }
                }

                break;
            }
            set_read_entry(&ui, rss::UNREAD_UUID, uuid.as_str());
        } else {
            set_read_entry(&ui, rss::UNREAD_UUID, uuid.as_str());
            set_read_entry(&ui, suuid.as_str(), uuid.as_str());
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_all_entry(move |suuid| {
        let ui = ui_handle.unwrap();

        if suuid != rss::FAVORITE_UUID {
            for rss in ui.global::<Store>().get_rss_lists().iter() {
                if rss.uuid != suuid {
                    continue;
                }

                for entry in rss.entry.iter() {
                    remove_entry(
                        &ui,
                        if suuid == rss::UNREAD_UUID {
                            entry.suuid.as_str()
                        } else {
                            rss::UNREAD_UUID
                        },
                        entry.uuid.as_str(),
                    );
                }

                break;
            }
        }
        remove_all_entry(&ui, suuid.as_str());
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_entry(move |suuid, uuid| {
        let ui = ui_handle.unwrap();
        if suuid == rss::UNREAD_UUID {
            for rss in ui.global::<Store>().get_rss_lists().iter() {
                if rss.uuid != suuid {
                    continue;
                }

                for entry in rss.entry.iter() {
                    if entry.uuid == uuid {
                        remove_entry(&ui, entry.suuid.as_str(), uuid.as_str());

                        break;
                    }
                }

                break;
            }
            remove_entry(&ui, rss::UNREAD_UUID, uuid.as_str());
        } else {
            if suuid != rss::FAVORITE_UUID {
                remove_entry(&ui, rss::UNREAD_UUID, uuid.as_str());
            }
            remove_entry(&ui, suuid.as_str(), uuid.as_str());
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_favorite_entry(move |suuid, uuid| {
        if suuid == rss::FAVORITE_UUID {
            return;
        }

        let ui = ui_handle.unwrap();

        for mut entry in ui.global::<Store>().get_rss_entry().iter() {
            if entry.uuid != uuid {
                continue;
            }

            if suuid != rss::UNREAD_UUID {
                if entry.tags.is_empty() {
                    entry.tags = get_rsslist(&ui, suuid.as_str()).name;
                } else {
                    entry.tags =
                        slint::format!("{},{}", get_rsslist(&ui, suuid.as_str()).name, entry.tags);
                }
            }
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
