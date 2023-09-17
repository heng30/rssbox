use super::data::SyncItem;
use super::entry;
use crate::db::data::{RssConfig, RssEntry};
use crate::slint_generatedAppWindow::{
    AppWindow, Logic, RssConfig as UIRssConfig, RssEntry as UIRssEntry, RssList, Store,
};
use crate::util::{crypto::md5_hex, http as uhttp, time as utime, translator::tr};
use crate::{config, db, CResult};
use atom_syndication::{Feed, FixedDateTime, Link, TextType};
use html2text;
use log::warn;
use rss::Channel;
use slint::{ComponentHandle, Model, ModelRc, VecModel, Weak};
use std::cmp::Ordering;
use std::io::BufReader;
use std::time::Duration;
use tokio::task::spawn;
use uuid::Uuid;

pub const UNREAD_UUID: &str = "unread-uuid";
pub const FAVORITE_UUID: &str = "favorite-uuid";

fn init_db(ui: &AppWindow) {
    for rss in ui.global::<Store>().get_rss_lists().iter() {
        let uuid = rss.uuid.as_str();

        match db::rss::is_exist(uuid) {
            Ok(exist) => {
                if exist {
                    continue;
                }
            }
            Err(e) => warn!("{:?}", e),
        }

        let config_json = match serde_json::to_string(&RssConfig::from(&rss)) {
            Ok(config) => config,
            Err(e) => {
                ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}{}: {:?}", tr("出错！"), &tr("原因"), e),
                    "warning".into(),
                );
                return;
            }
        };

        if let Err(e) = db::rss::insert(uuid, &config_json) {
            ui.global::<Logic>().invoke_show_message(
                slint::format!("{}{}: {:?}", tr("出错！"), tr("原因"), e),
                "warning".into(),
            );
            return;
        }
    }
}

fn init_rss(ui: &AppWindow) {
    match db::rss::select_all() {
        Ok(items) => {
            let mut rsslists = vec![];
            let mut unread_entrys = vec![];

            for item in items.into_iter() {
                let mut rss = RssList {
                    entry: ModelRc::new(VecModel::from(entry::get_from_db(&item.0.as_str()))),
                    uuid: item.0.into(),
                    ..Default::default()
                };

                match serde_json::from_str::<RssConfig>(&item.1) {
                    Ok(conf) => {
                        rss.is_mark = conf.is_mark;
                        rss.use_proxy = conf.use_proxy;
                        rss.icon_index = conf.icon_index;
                        rss.name = conf.name.into();
                        rss.url = conf.url.into();
                        rss.update_time = conf.update_time.into();
                        rss.feed_format = conf.feed_format.into();
                    }
                    Err(e) => {
                        warn!("{:?}", e);
                        continue;
                    }
                }

                for mut entry in rss.entry.iter() {
                    if !entry.is_read {
                        entry.tags = if entry.tags.is_empty() {
                            rss.name.as_str().into()
                        } else {
                            slint::format!("{},{}", rss.name, entry.tags)
                        };
                        unread_entrys.push(entry);
                    }
                }

                if rss.uuid == UNREAD_UUID {
                    ui.global::<Store>().set_rss_entry(rss.entry.clone());
                }

                rsslists.push(rss);
            }

            rsslists.sort_by(|a, b| -> Ordering {
                if a.uuid == UNREAD_UUID {
                    Ordering::Less
                } else if b.uuid == UNREAD_UUID {
                    Ordering::Greater
                } else if a.uuid == FAVORITE_UUID {
                    Ordering::Less
                } else if b.uuid == FAVORITE_UUID {
                    Ordering::Greater
                } else if a.is_mark && b.is_mark {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                } else if a.is_mark && !b.is_mark {
                    Ordering::Less
                } else if !a.is_mark && b.is_mark {
                    Ordering::Greater
                } else {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                }
            });

            ui.global::<Store>()
                .get_rss_entry()
                .as_any()
                .downcast_ref::<VecModel<UIRssEntry>>()
                .expect("We know we set a VecModel earlier")
                .set_vec(unread_entrys);

            ui.global::<Store>()
                .set_rss_lists(ModelRc::new(VecModel::from(rsslists)));
        }
        Err(e) => {
            warn!("{:?}", e);
        }
    }
}

pub fn init(ui: &AppWindow) {
    init_db(ui);
    init_rss(ui);

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_set_rss_dialog(move |uuid| {
        let ui = ui_handle.unwrap();

        for rss in ui.global::<Store>().get_rss_lists().iter() {
            if rss.uuid == uuid {
                ui.invoke_rss_dialog_set(UIRssConfig {
                    uuid: uuid,
                    name: rss.name,
                    url: rss.url,
                    use_proxy: rss.use_proxy,
                    icon_index: rss.icon_index,
                    feed_format: rss.feed_format,
                });
                return;
            }
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_reset_rss_dialog(move || {
        let ui = ui_handle.unwrap();
        ui.invoke_rss_dialog_set(UIRssConfig::default());
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_new_rss(move |config| {
        let ui = ui_handle.unwrap();

        let mut rss: RssList = config.into();
        rss.uuid = Uuid::new_v4().to_string().into();
        rss.entry = ModelRc::new(VecModel::default());

        match serde_json::to_string(&RssConfig::from(&rss)) {
            Ok(config) => {
                if let Err(e) = db::rss::insert(rss.uuid.as_str(), &config) {
                    ui.global::<Logic>().invoke_show_message(
                        slint::format!("{}{}: {:?}", tr("新建失败！"), tr("原因"), e),
                        "warning".into(),
                    );
                    return;
                }

                if let Err(e) = db::entry::new(rss.uuid.as_str()) {
                    ui.global::<Logic>().invoke_show_message(
                        slint::format!("{}{}: {:?}", tr("新建失败！"), tr("原因"), e),
                        "warning".into(),
                    );
                    return;
                }
            }
            Err(e) => {
                ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}{}: {:?}", tr("新建失败！"), tr("原因"), e),
                    "warning".into(),
                );
                return;
            }
        };

        ui.global::<Store>()
            .get_rss_lists()
            .as_any()
            .downcast_ref::<VecModel<RssList>>()
            .expect("We know we set a VecModel earlier")
            .push(rss);

        ui.global::<Logic>()
            .invoke_show_message(tr("新建成功！").into(), "success".into());
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_save_rss(move |uuid, config| {
        let ui = ui_handle.unwrap();

        for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
            if rss.uuid != uuid {
                continue;
            }

            rss.name = config.name;
            rss.url = config.url;
            rss.use_proxy = config.use_proxy;
            rss.icon_index = config.icon_index;
            rss.feed_format = config.feed_format;

            match serde_json::to_string(&RssConfig::from(&rss)) {
                Ok(config) => {
                    if let Err(e) = db::rss::update(uuid.as_str(), &config) {
                        ui.global::<Logic>().invoke_show_message(
                            slint::format!("{}{}: {:?}", tr("保存失败！"), tr("原因"), e),
                            "warning".into(),
                        );
                        return;
                    } else {
                        ui.global::<Logic>()
                            .invoke_show_message(tr("保存成功！").into(), "success".into());
                    }
                }
                Err(e) => {
                    ui.global::<Logic>().invoke_show_message(
                        slint::format!("{}{}: {:?}", tr("保存失败！"), tr("原因"), e),
                        "warning".into(),
                    );
                    return;
                }
            };

            ui.global::<Store>()
                .get_rss_lists()
                .set_row_data(index, rss);
            return;
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_delete_rss(move |uuid| {
        let ui = ui_handle.unwrap();

        if uuid == UNREAD_UUID || uuid == FAVORITE_UUID {
            ui.global::<Logic>()
                .invoke_show_message(tr("不允许删！").into(), "warning".into());
            return;
        }

        for (index, rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
            if rss.uuid != uuid {
                continue;
            }

            ui.global::<Logic>().invoke_remove_all_entry(uuid.clone());
            ui.global::<Logic>()
                .invoke_switch_rss(uuid.clone(), UNREAD_UUID.into());

            if let Err(e) = db::entry::drop_table(uuid.as_str()) {
                ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}{}: {:?}", tr("删除失败！"), tr("原因"), e),
                    "warning".into(),
                );
            }

            ui.global::<Store>()
                .get_rss_lists()
                .as_any()
                .downcast_ref::<VecModel<RssList>>()
                .expect("We know we set a VecModel earlier")
                .remove(index);

            match db::rss::delete(uuid.as_str()) {
                Err(e) => {
                    ui.global::<Logic>().invoke_show_message(
                        slint::format!("{}{}: {:?}", tr("删除会话失败！"), tr("原因"), e),
                        "warning".into(),
                    );
                }
                _ => {
                    ui.global::<Logic>()
                        .invoke_show_message(tr("删除会话成功！").into(), "success".into());
                }
            }

            return;
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_toggle_rss_mark(move |index, uuid| {
        let ui = ui_handle.unwrap();
        let index = index as usize;

        if let Some(mut rss) = ui.global::<Store>().get_rss_lists().row_data(index) {
            rss.is_mark = !rss.is_mark;

            match serde_json::to_string(&RssConfig::from(&rss)) {
                Ok(config) => {
                    if let Err(e) = db::rss::update(uuid.as_str(), &config) {
                        ui.global::<Logic>().invoke_show_message(
                            slint::format!("{}{}: {:?}", tr("保存失败！"), tr("原因"), e),
                            "warning".into(),
                        );
                        return;
                    }
                }
                Err(e) => {
                    ui.global::<Logic>().invoke_show_message(
                        slint::format!("{}{}: {:?}", tr("保存失败！"), tr("原因"), e),
                        "warning".into(),
                    );
                    return;
                }
            };

            ui.global::<Store>()
                .get_rss_lists()
                .set_row_data(index, rss)
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_switch_rss(move |old_uuid, new_uuid| {
            if old_uuid == new_uuid || new_uuid.is_empty() {
                return;
            }

            let ui = ui_handle.unwrap();
            let entry = ui.global::<Store>().get_rss_entry();

            let mut index = 0;
            for (row, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
                if rss.uuid == old_uuid {
                    rss.entry = entry.clone();
                    ui.global::<Store>().get_rss_lists().set_row_data(row, rss);
                    index += 1;
                } else if rss.uuid == new_uuid {
                    ui.global::<Store>().set_rss_entry(rss.entry);
                    ui.global::<Store>().set_current_rss_uuid(new_uuid.clone());
                    index += 1;
                }

                if index == 2 {
                    break;
                }
            }

            ui.invoke_scroll_to_top();
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_sync_rss(move |suuid| {
        let ui = ui_handle.unwrap();
        if suuid == FAVORITE_UUID {
            ui.global::<Logic>()
                .invoke_show_message(tr("不允许刷新！").into(), "warning".into());
            return;
        }

        let mut items: Vec<SyncItem> = vec![];

        for rss in ui.global::<Store>().get_rss_lists().iter() {
            if rss.uuid == UNREAD_UUID || rss.uuid == FAVORITE_UUID {
                continue;
            }

            if suuid == UNREAD_UUID {
                items.push(rss.into());
            } else if suuid == rss.uuid {
                items.push(rss.into());
                break;
            }
        }

        let ui_handle = ui.as_weak();
        ui.global::<Logic>()
            .invoke_show_message("正在同步...".into(), "info".into());
        spawn(async move {
            if let Err(e) = sync_rss(ui_handle, items).await {
                warn!("{:?}", e);
            }
        });
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_toggle_unread_count_flag(move |suuid| {
            let ui = ui_handle.unwrap();
            ui.global::<Store>().invoke_toggle_unread_count_flag();

            for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
                if rss.uuid == suuid {
                    rss.unread_counts_flag = !rss.unread_counts_flag;
                    ui.global::<Store>()
                        .get_rss_lists()
                        .set_row_data(index, rss);
                    return;
                }
            }
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_get_update_time(move |suuid, _flag| {
            let ui = ui_handle.unwrap();

            for rss in ui.global::<Store>().get_rss_lists().iter() {
                if rss.uuid == suuid {
                    return rss.update_time;
                }
            }
            "".into()
        });
}

fn update_new_entrys(ui: &AppWindow, suuid: String, entrys: Vec<RssEntry>) {
    let mut unread_entrys = ModelRc::default();
    for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
        if rss.uuid == UNREAD_UUID {
            unread_entrys = rss.entry.clone();

            rss.update_time = utime::local_now("%Y-%m-%d %H:%M").into();
            ui.global::<Store>()
                .get_rss_lists()
                .set_row_data(index, rss);
            ui.global::<Store>().invoke_toggle_update_time_flag();

            continue;
        }

        if rss.uuid != suuid {
            continue;
        }

        for mut entry in entrys.into_iter() {
            let mut found = false;
            for item in rss.entry.iter() {
                if item.url == entry.url {
                    found = true;
                    break;
                }
            }
            if !found {
                entry::update_new_entry(ui, &suuid, &suuid, entry.clone());
            }

            found = false;
            for item in unread_entrys.iter() {
                if item.url == entry.url {
                    found = true;
                    break;
                }
            }
            if !found {
                entry.tags = if entry.tags.is_empty() {
                    rss.name.as_str().into()
                } else {
                    format!("{},{}", rss.name, entry.tags)
                };
                entry::update_new_entry(ui, &suuid, UNREAD_UUID, entry);
            }
        }

        rss.update_time = utime::local_now("%Y-%m-%d %H:%M").into();
        ui.global::<Store>()
            .get_rss_lists()
            .set_row_data(index, rss);
        ui.global::<Store>().invoke_toggle_update_time_flag();

        return;
    }
}

// Be careful, It runs in another thread
async fn fetch_entry(config: SyncItem) -> Result<Vec<RssEntry>, Box<dyn std::error::Error>> {
    let rss_config = config::rss();
    let request_timeout = u64::min(rss_config.sync_timeout as u64, 10_u64);

    let client = uhttp::client(config.use_proxy)?;
    let content = client
        .get(&config.url)
        .headers(uhttp::headers())
        .timeout(Duration::from_secs(request_timeout))
        .send()
        .await?
        .bytes()
        .await?;

    let mut entry = vec![];

    let feed_format = config.feed_format.to_lowercase();

    if feed_format == "rss" {
        let ch = Channel::read_from(&content[..])?;
        for item in ch.items() {
            let url = item.link().unwrap_or("").to_string();
            let title = item.title().unwrap_or("").to_string();
            let author = item.author().unwrap_or("").to_string();
            let pub_date = item.pub_date().unwrap_or("").to_string();

            let summary = if item.description().is_some() {
                let s = item.description().unwrap();
                html2text::from_read(s.as_bytes(), usize::MAX)
                    .trim()
                    .to_string()
            } else {
                "".to_string()
            };

            let tags = item
                .categories()
                .iter()
                .map(|c| c.name().to_string())
                .collect::<Vec<_>>()
                .join(",")
                .to_string();

            if url.is_empty() || title.is_empty() {
                continue;
            }

            entry.push(RssEntry {
                url,
                title,
                pub_date,
                author,
                summary,
                tags,
                uuid: Uuid::new_v4().to_string(),
                ..Default::default()
            });
        }
    } else if feed_format == "atom" {
        let feed = Feed::read_from(BufReader::new(&content[..]))?;
        for item in feed.entries() {
            let url = item
                .links()
                .first()
                .unwrap_or(&Link::default())
                .href()
                .to_string();
            let title = item.title().as_str().to_string();
            let pub_date = item
                .published()
                .unwrap_or(&FixedDateTime::default())
                .to_string();

            let author = item
                .authors()
                .iter()
                .map(|p| p.name().to_string())
                .collect::<Vec<_>>()
                .join("|")
                .to_string();

            let summary = if item.summary().is_some() {
                let s = item.summary().unwrap();
                if s.r#type == TextType::Text {
                    s.as_str().to_string()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            };

            let tags = item
                .categories()
                .iter()
                .map(|c| c.term().to_string())
                .collect::<Vec<_>>()
                .join(",")
                .to_string();

            if url.is_empty() || title.is_empty() {
                continue;
            }

            entry.push(RssEntry {
                url,
                title,
                pub_date,
                author,
                summary,
                tags,
                uuid: Uuid::new_v4().to_string(),
                ..Default::default()
            });
        }
    } else {
        warn!("unimplemented feed format: {}", feed_format);
    }

    entry = entry
        .into_iter()
        .filter(|e| match db::trash::is_exist(&md5_hex(e.url.as_str())) {
            Ok(flag) => !flag,
            _ => true,
        })
        .rev()
        .collect();

    Ok(entry)
}

// Be careful, It runs in another thread
pub async fn sync_rss(ui: Weak<AppWindow>, items: Vec<SyncItem>) -> CResult {
    let mut is_success = true;
    for item in items.into_iter() {
        let suuid = item.uuid.clone();
        match fetch_entry(item).await {
            Ok(entry) => {
                let ui = ui.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    let ui = ui.unwrap();
                    update_new_entrys(&ui, suuid, entry);
                });
            }
            Err(e) => {
                is_success = false;
                let err = format!("{:?}", e);
                let ui = ui.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    let ui = ui.unwrap();
                    ui.global::<Logic>().invoke_show_message(
                        slint::format!("{} {}: {}", tr("同步失败！"), tr("原因"), err),
                        "warning".into(),
                    );
                });
            }
        }
    }

    if is_success {
        let ui = ui.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let ui = ui.unwrap();
            ui.global::<Logic>()
                .invoke_show_message(tr("同步成功！").into(), "success".into());
        });
    }

    Ok(())
}
