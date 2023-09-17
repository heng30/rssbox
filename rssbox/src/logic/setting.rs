use crate::slint_generatedAppWindow::{AppWindow, Logic, Store};
use crate::util::translator::tr;
use crate::{config, db};
use slint::{ComponentHandle, Weak};

pub fn init(ui: &AppWindow) {
    init_setting_dialog(ui.as_weak());

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_setting_cancel(move || {
        init_setting_dialog(ui_handle.clone());
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_setting_ok(move |setting_config| {
        let ui = ui_handle.unwrap();
        let mut config = config::config();

        config.ui.font_size = setting_config
            .ui
            .font_size
            .to_string()
            .parse()
            .unwrap_or(20);
        config.ui.win_width = setting_config
            .ui
            .win_width
            .to_string()
            .parse()
            .unwrap_or(1200);
        config.ui.win_height = setting_config
            .ui
            .win_height
            .to_string()
            .parse()
            .unwrap_or(800);

        config.ui.font_family = setting_config.ui.font_family.to_string();
        config.ui.language = setting_config.ui.language.to_string();

        config.rss.sync_interval = setting_config.rss.sync_interval as u32;
        config.rss.sync_interval_enabled = setting_config.rss.sync_interval_enabled;
        config.rss.sync_timeout = setting_config.rss.sync_timeout as u32;
        config.rss.browser = setting_config.rss.browser.to_string();
        config.rss.start_sync = setting_config.rss.start_sync;

        config.socks5.url = setting_config.proxy.url.to_string();
        config.socks5.port = setting_config
            .proxy
            .port
            .to_string()
            .parse()
            .unwrap_or(1080);

        match config::save(config) {
            Err(e) => {
                ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}{:?}", tr("保存失败") + "！", e),
                    "warning".into(),
                );
            }
            _ => {
                init_setting_dialog(ui.as_weak());
                ui.global::<Logic>()
                    .invoke_show_message((tr("保存成功") + "!").into(), "success".into());
            }
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_clear_trash_box(move || {
        let ui = ui_handle.unwrap();
        let mut setting_dialog = ui.global::<Store>().get_setting_dialog_config();
        setting_dialog.rss.trash_count = 0;
        ui.global::<Store>()
            .set_setting_dialog_config(setting_dialog);
        let _ = db::trash::delete_all();

        ui.global::<Logic>()
            .invoke_show_message("清空成功！".into(), "success".into());
    });
}

fn init_setting_dialog(ui: Weak<AppWindow>) {
    let ui = ui.unwrap();
    let ui_config = config::ui();
    let rss_config = config::rss();
    let socks5_config = config::socks5();

    let mut setting_dialog = ui.global::<Store>().get_setting_dialog_config();
    setting_dialog.ui.font_size = slint::format!("{}", ui_config.font_size);
    setting_dialog.ui.font_family = ui_config.font_family.into();
    setting_dialog.ui.win_width = slint::format!("{}", ui_config.win_width);
    setting_dialog.ui.win_height = slint::format!("{}", ui_config.win_height);
    setting_dialog.ui.language = ui_config.language.into();

    setting_dialog.rss.sync_interval = rss_config.sync_interval as i32;
    setting_dialog.rss.sync_interval_enabled = rss_config.sync_interval_enabled;
    setting_dialog.rss.sync_timeout = rss_config.sync_timeout as i32;
    setting_dialog.rss.browser = rss_config.browser.into();
    setting_dialog.rss.start_sync = rss_config.start_sync;
    setting_dialog.rss.trash_count = db::trash::row_count().unwrap_or(0_i32);

    setting_dialog.proxy.url = socks5_config.url.into();
    setting_dialog.proxy.port = slint::format!("{}", socks5_config.port);

    ui.global::<Store>()
        .set_setting_dialog_config(setting_dialog);
}
