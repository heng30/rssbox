use crate::db;
use crate::db::data::RssEntry;
use crate::rss;
use crate::slint_generatedAppWindow::{AppWindow, Logic, RssEntry as UIRssEntry, Store};
use crate::util::translator::tr;
use log::warn;
use slint::{ComponentHandle, Model, ModelExt, ModelRc, SharedString, VecModel};

pub fn init(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_parse_tags(move |tags| {
        let items: Vec<_> = tags
            .split(',')
            .map(|tag| SharedString::from(tag.trim()))
            .collect();

        ModelRc::new(VecModel::from(items))
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_set_read_all_entry(move || {
        let ui = ui_handle.unwrap();
        let suuid = ui.global::<Store>().get_current_rss_uuid();

        for (index, mut entry) in ui.global::<Store>().get_rss_entry().iter().enumerate() {
            if entry.is_read {
                continue;
            }

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
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_set_read_entry(move |uuid| {
        let ui = ui_handle.unwrap();
        let suuid = ui.global::<Store>().get_current_rss_uuid();

        for (index, mut entry) in ui.global::<Store>().get_rss_entry().iter().enumerate() {
            if entry.is_read || entry.uuid != uuid {
                continue;
            }

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
            return;
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_all_entry(move |suuid| {
        let ui = ui_handle.unwrap();

        ui.global::<Store>().set_rss_entry(ModelRc::default());

        if let Err(e) = db::entry::drop_table(suuid.as_str()) {
            ui.global::<Logic>().invoke_show_message(
                slint::format!("{}{}: {:?}", tr("出错！"), tr("原因"), e),
                "warning".into(),
            );
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_entry(move |uuid| {
        let ui = ui_handle.unwrap();
        let suuid = ui.global::<Store>().get_current_rss_uuid();

        for (index, entry) in ui.global::<Store>().get_rss_entry().iter().enumerate() {
            if entry.uuid != uuid {
                continue;
            }

            if let Err(e) = db::entry::delete(suuid.as_str(), entry.uuid.as_str()) {
                ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}{}: {:?}", tr("删除失败！"), tr("原因"), e),
                    "warning".into(),
                );
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
    ui.global::<Logic>().on_favorite_entry(move |uuid| {
        let ui = ui_handle.unwrap();
        let suuid = ui.global::<Store>().get_current_rss_uuid();

        for entry in ui.global::<Store>().get_rss_entry().iter() {
            if entry.uuid != uuid {
                continue;
            }

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

    // callback sync-rss();
}
