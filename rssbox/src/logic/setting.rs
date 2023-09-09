use crate::config;
use crate::slint_generatedAppWindow::{AppWindow, Logic, Store};
use crate::util::{self, translator::tr};
use log::warn;
use slint::{ComponentHandle, VecModel, Weak};
use std::rc::Rc;

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
}

fn init_setting_dialog(ui: Weak<AppWindow>) {
    let ui = ui.unwrap();
    let ui_config = config::ui();
    let socks5_config = config::socks5();

    let mut setting_dialog = ui.global::<Store>().get_setting_dialog_config();
    setting_dialog.ui.font_size = slint::format!("{}", ui_config.font_size);
    setting_dialog.ui.font_family = ui_config.font_family.into();
    setting_dialog.ui.win_width = slint::format!("{}", ui_config.win_width);
    setting_dialog.ui.win_height = slint::format!("{}", ui_config.win_height);
    setting_dialog.ui.language = ui_config.language.into();

    setting_dialog.proxy.url = socks5_config.url.into();
    setting_dialog.proxy.port = slint::format!("{}", socks5_config.port);

    ui.global::<Store>()
        .set_setting_dialog_config(setting_dialog);
}
