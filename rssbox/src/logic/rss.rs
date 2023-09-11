use super::data::SyncItem;
use super::entry;
use crate::db;
use crate::db::data::{RssConfig, RssEntry};
use crate::slint_generatedAppWindow::{AppWindow, Logic, RssConfig as UIRssConfig, RssList, Store};
use crate::util::http as uhttp;
use crate::util::translator::tr;
use crate::CResult;
use log::warn;
use rss::Channel;
use slint::{ComponentHandle, Model, ModelExt, ModelRc, VecModel, Weak};
use std::cmp::Ordering;
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
            let rsslists = VecModel::default();

            for item in items.into_iter() {
                let config = item.1;

                let mut rss = RssList {
                    entry: ModelRc::new(VecModel::from(entry::get_from_db(&item.0.as_str()))),
                    uuid: item.0.into(),
                    ..Default::default()
                };

                if rss.uuid == UNREAD_UUID {
                    ui.global::<Store>().set_rss_entry(rss.entry.clone());
                }

                match serde_json::from_str::<RssConfig>(&config) {
                    Ok(conf) => {
                        rss.is_mark = conf.is_mark;
                        rss.use_proxy = conf.use_proxy;
                        rss.icon_index = conf.icon_index;
                        rss.name = conf.name.into();
                        rss.url = conf.url.into();
                        rss.update_time = conf.update_time.into();
                    }
                    Err(e) => {
                        warn!("{:?}", e);
                        continue;
                    }
                }
                rsslists.push(rss);
            }

            let rsslists = rsslists.sort_by(|a, b| -> Ordering {
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
                .set_rss_lists(ModelRc::new(VecModel::from(
                    rsslists.iter().collect::<Vec<_>>(),
                )));
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
                    ui.global::<Store>()
                        .set_current_rss_uuid(UNREAD_UUID.into());
                    ui.global::<Logic>()
                        .invoke_show_message(tr("删除会话成功！").into(), "success".into());
                }
            }

            if let Err(e) = db::entry::drop_table(rss.uuid.as_str()) {
                ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}{}: {:?}", tr("删除失败！"), tr("原因"), e),
                    "warning".into(),
                );
            }

            // TODO: remove rss-list-items

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
        spawn(async move {
            if let Err(e) = sync_rss(ui_handle, items).await {
                warn!("{:?}", e);
            }
        });
    });
}

fn update_new_entry(ui: &AppWindow, suuid: String, entry: Vec<RssEntry>) {}

async fn fetch_entry(config: SyncItem) -> Result<Vec<RssEntry>, Box<dyn std::error::Error>> {
    let request_timeout = 10;

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
    let ch = Channel::read_from(&content[..])?;
    for item in ch.items() {
        entry.push(RssEntry {
            uuid: Uuid::new_v4().to_string(),
            url: item.link().unwrap_or("").to_string(),
            title: item.title().unwrap_or("").to_string(),
            pub_date: item.pub_date().unwrap_or("").to_string(),
            ..Default::default()
        });
    }

    Ok(entry)
}

// Be careful, It run in another thread
pub async fn sync_rss(ui: Weak<AppWindow>, items: Vec<SyncItem>) -> CResult {
    for item in items.into_iter() {
        let suuid = item.uuid.clone();
        match fetch_entry(item).await {
            Ok(entry) => {
                warn!("{:?}", entry);
                let ui = ui.clone();
                if let Err(e) = slint::invoke_from_event_loop(move || {
                    let ui = ui.unwrap();
                    update_new_entry(&ui, suuid, entry);
                }) {
                    warn!("{:?}", e);
                }
            }
            Err(e) => {
                warn!("{:?}", e);
            }
        }
    }
    Ok(())
}
