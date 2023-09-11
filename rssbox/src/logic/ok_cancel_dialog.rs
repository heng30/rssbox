use crate::slint_generatedAppWindow::{AppWindow, Logic, Store};
use slint::ComponentHandle;

pub fn init(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_handle_ok_cancel_dialog(move |handle_type, handle_uuid| {
            let ui = ui_handle.unwrap();

            if handle_type.as_str() == "rss" {
                ui.global::<Logic>().invoke_delete_rss(handle_uuid);
            } else if handle_type.as_str() == "rss-all-entry" {
                ui.global::<Logic>().invoke_remove_all_entry(handle_uuid);
            } else if handle_type.as_str() == "rss-entry" {
                let suuid = ui.global::<Store>().get_current_rss_uuid();
                ui.global::<Logic>().invoke_remove_entry(suuid, handle_uuid);
            // } else if handle_type.as_str() == "session-archive-item" {
            //     ui.global::<Logic>().invoke_delete_session_archive(
            //         ui.global::<Store>().get_current_session_uuid(),
            //         handle_uuid,
            //     );
            }
        });
}
