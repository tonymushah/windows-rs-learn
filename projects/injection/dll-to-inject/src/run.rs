use eframe::{App, AppCreator, egui};
use windows::Win32::Foundation::HMODULE;

use crate::utils::close_frame_window::close_frame_window;

#[derive(Debug, Default)]
struct EframeApp {}

impl EframeApp {
    fn app_creator_default<'a>() -> AppCreator<'a> {
        Box::new(|_cc| Ok(Box::new(Self::default())))
    }
}
impl App for EframeApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Injected DLL");
            if ui.button("Close").clicked() {
                if let Err(err) = close_frame_window(frame) {
                    log::error!("{err}");
                }
            }
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
