pub mod dis_log;

use dis_log::log_widget;
use eframe::{App, AppCreator, egui};
use windows::Win32::Foundation::HMODULE;

// use crate::utils::close_frame_window::close_frame_window;

#[derive(Debug, Default)]
struct EframeApp {}

impl EframeApp {
    fn app_creator_default<'a>() -> AppCreator<'a> {
        Box::new(|_cc| Ok(Box::new(Self::default())))
    }
}
impl App for EframeApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Injected DLL");
            log_widget(ui);
        });
    }
}

pub fn run(_hmodule: HMODULE) -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions {
        event_loop_builder: Some(Box::new(|_ctx| {
            #[cfg(target_os = "windows")]
            {
                use winit::platform::windows::EventLoopBuilderExtWindows;
                _ctx.with_any_thread(true);
            }
        })),
        run_and_return: true,
        ..Default::default()
    };
    eframe::run_native(
        "Injected DLL",
        native_options,
        EframeApp::app_creator_default(),
    )
    .map_err(|err| anyhow::anyhow!("{err}"))?;
    Ok(())
}
