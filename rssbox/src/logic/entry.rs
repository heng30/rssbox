use crate::db;
use crate::db::data::RssEntry;
use crate::rss;
use crate::slint_generatedAppWindow::{AppWindow, Logic, RssEntry as UIRssEntry, Store};
use crate::util::translator::tr;
use log::warn;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

pub fn get_from_db(suuid: &str) -> Vec<UIRssEntry> {
    match db::entry::select_all(suuid) {
        Ok(items) => items
            .iter()
            .rev()
            .map(|item| match serde_json::from_str::<RssEntry>(&item.1) {
                Ok(entry) => entry.into(),
                Err(e) => {
                    warn!("{:?}", e);
                    UIRssEntry::default()
                }
            })
            .collect::<Vec<_>>(),
        Err(e) => {
            warn!("{:?}", e);
            vec![]
        }
    }
}

pub fn update_entry(ui: &AppWindow, suuid: &str, entry: RssEntry) {
    if suuid == rss::UNREAD_UUID {
        return;
    }

    match serde_json::to_string(&entry) {
        Ok(data) => {
            if let Err(e) = db::entry::insert(suuid, entry.uuid.as_str(), &data) {
                warn!("{:?}", e);
                return;
            }

            let cur_suuid = ui.global::<Store>().get_current_rss_uuid();

            for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
                if rss.uuid.as_str() != suuid {
                    continue;
                }

                if suuid == cur_suuid.as_str() {
                    let mut entrys = ui
                        .global::<Store>()
                        .get_rss_entry()
                        .iter()
                        .collect::<Vec<_>>();
                    entrys.insert(0, entry.into());
                    ui.global::<Store>()
                        .set_rss_entry(ModelRc::new(VecModel::from(entrys)));
                } else {
                    let mut entrys = rss.entry.iter().collect::<Vec<_>>();
                    entrys.insert(0, entry.into());
                    rss.entry = ModelRc::new(VecModel::from(entrys));

                    ui.global::<Store>()
                        .get_rss_lists()
                        .set_row_data(index, rss);
                }

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

    let set_read =
        |ui: &AppWindow, suuid: &slint::SharedString, index: usize, mut entry: UIRssEntry| {
            entry.is_read = true;

            match serde_json::to_string(&RssEntry::from(&entry)) {
                Ok(data) => {
                    if let Err(e) = db::entry::update(suuid.as_str(), entry.uuid.as_str(), &data) {
                        ui.global::<Logic>().invoke_show_message(
                            slint::format!("{}{}: {:?}", tr("保存失败！"), tr("原因"), e),
                            "warning".into(),
                        );
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

        ui.global::<Store>().set_rss_entry(ModelRc::default());

        if let Err(e) = db::entry::delete_all(suuid.as_str()) {
            ui.global::<Logic>().invoke_show_message(
                slint::format!("{}{}: {:?}", tr("清空失败！"), tr("原因"), e),
                "warning".into(),
            );
        } else {
            ui.global::<Logic>()
                .invoke_show_message(tr("清空成功！").into(), "success".into());
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_entry(move |suuid, uuid| {
        let ui = ui_handle.unwrap();

        for (index, entry) in ui.global::<Store>().get_rss_entry().iter().enumerate() {
            if entry.uuid != uuid {
                continue;
            }

            if let Err(e) = db::entry::delete(suuid.as_str(), entry.uuid.as_str()) {
                ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}{}: {:?}", tr("删除失败！"), tr("原因"), e),
                    "warning".into(),
                );
            } else {
                ui.global::<Logic>()
                    .invoke_show_message(tr("删除成功！").into(), "success".into());
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

        for entry in ui.global::<Store>().get_rss_entry().iter() {
            if entry.uuid != uuid {
                continue;
            }

            // TODO: set tag
            // entry.tags = rss

            match serde_json::to_string(&RssEntry::from(&entry)) {
                Ok(data) => {
                    if let Err(e) =
                        db::entry::insert(rss::FAVORITE_UUID, entry.uuid.as_str(), &data)
                    {
                        ui.global::<Logic>().invoke_show_message(
                            slint::format!("{}{}: {:?}", tr("保存失败！"), tr("原因"), e),
                            "warning".into(),
                        );
                        return;
                    }

                    for rss in ui.global::<Store>().get_rss_lists().iter() {
                        if rss.uuid != rss::FAVORITE_UUID {
                            continue;
                        }

                        rss.entry
                            .as_any()
                            .downcast_ref::<VecModel<UIRssEntry>>()
                            .expect("We know we set a VecModel earlier")
                            .push(entry);

                        break;
                    }

                    ui.global::<Logic>()
                        .invoke_show_message(tr("收藏成功！").into(), "success".into());
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
