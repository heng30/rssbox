slint::include_modules!();

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

use chrono::Local;
use chrono::Utc;
use env_logger::fmt::Color as LColor;
use log::debug;
use slint::{Timer, TimerMode};
use std::env;
use std::io::Write;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

mod config;
mod db;
mod logic;
mod util;
mod version;

use logic::{about, clipboard, entry, message, ok_cancel_dialog, rss, setting, window};

pub type CResult = Result<(), Box<dyn std::error::Error>>;

static SYNC_TIMESTAMP_CACHE: AtomicI64 = AtomicI64::new(0);

#[tokio::main]
async fn main() -> CResult {
    init_logger();
    debug!("{}", "start...");

    config::init();
    db::init();

    let ui = AppWindow::new().unwrap();
    logic::util::init(&ui);

    clipboard::init(&ui);
    message::init(&ui);
    window::init(&ui);
    about::init(&ui);
    setting::init(&ui);
    rss::init(&ui);
    entry::init(&ui);
    ok_cancel_dialog::init(&ui);

    let _timer = sync_rss(&ui);
    ui.run().unwrap();

    debug!("{}", "exit...");
    Ok(())
}

fn sync_rss(ui: &AppWindow) -> Timer {
    let rss_config = config::rss();
    if rss_config.start_sync {
        ui.global::<Logic>()
            .invoke_sync_rss(rss::UNREAD_UUID.into());
    }

    let ui_handle = ui.as_weak();
    SYNC_TIMESTAMP_CACHE.store(Utc::now().timestamp(), Ordering::SeqCst);

    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_secs(10), move || {
        let config = config::rss();
        let sync_interval = i64::max(config.sync_interval as i64, 1_i64) * 60;
        let now = Utc::now().timestamp();
        if SYNC_TIMESTAMP_CACHE.load(Ordering::SeqCst) + sync_interval < now {
            SYNC_TIMESTAMP_CACHE.store(now, Ordering::SeqCst);

            let ui = ui_handle.unwrap();
            ui.global::<Logic>()
                .invoke_sync_rss(rss::UNREAD_UUID.into());
        }
    });
    timer
}

fn init_logger() {
    env_logger::builder()
        .format(|buf, record| {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
            let mut level_style = buf.style();
            match record.level() {
                log::Level::Warn | log::Level::Error => {
                    level_style.set_color(LColor::Red).set_bold(true)
                }
                _ => level_style.set_color(LColor::Blue).set_bold(true),
            };

            writeln!(
                buf,
                "[{} {} {} {}] {}",
                ts,
                level_style.value(record.level()),
                record
                    .file()
                    .unwrap_or("None")
                    .split('/')
                    .last()
                    .unwrap_or("None"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();
}
