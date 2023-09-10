use crate::slint_generatedAppWindow::{AppWindow, Logic, Store};
use slint::{ComponentHandle, Model, ModelExt, ModelRc, SharedString, VecModel};

pub fn init(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_parse_tags(move |tags| {
        let items: Vec<_> = tags.split(',').map(|tag| SharedString::from(tag.trim())).collect();

        ModelRc::new(VecModel::from(items))
    });
}
