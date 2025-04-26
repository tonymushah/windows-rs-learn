use std::{collections::VecDeque, sync::LazyLock, time::SystemTime};

use eframe::egui::{self, Context, mutex::RwLock};
use fern::Output;

const MAXIMUM_CAPACITY: usize = 200;

static LOG_DATA: LazyLock<RwLock<VecDeque<String>>> =
    LazyLock::new(|| RwLock::new(VecDeque::with_capacity(MAXIMUM_CAPACITY)));

pub fn output() -> Output {
    Output::call(|rec| {
        let mut write = LOG_DATA.write();
        if write.len() >= MAXIMUM_CAPACITY - 1 {
            write.pop_front();
        }
        write.push_back(format!(
            "[{} {} {}] {}",
            humantime::format_rfc3339_seconds(SystemTime::now()),
            rec.level(),
            rec.target(),
            rec.args()
        ));
    })
}

pub(super) fn log_widget(ctx: &Context) {
    egui::Window::new("Fern Logging")
        .vscroll(true)
        .max_height(500.0)
        .show(ctx, |ui| {
            let read = LOG_DATA.read();
            for entry in read.iter() {
                ui.label(entry);
            }
        });
}
