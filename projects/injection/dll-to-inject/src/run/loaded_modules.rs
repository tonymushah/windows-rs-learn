use std::{fmt::Debug, os::raw::c_void, sync::LazyLock};

use eframe::egui::{Widget, mutex::RwLock};
use windows::Win32::System::Threading::{
    CreateThreadpoolWork, GetCurrentProcessId, PTP_CALLBACK_INSTANCE, PTP_WORK,
    SubmitThreadpoolWork,
};
use windows_core::Owned;

use crate::utils::get_loaded_module::{ModuleEntry, modules};

pub struct LoadedModules {
    work: Owned<PTP_WORK>,
}

impl Debug for LoadedModules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedModules")
            .field("work", &self.work)
            .finish()
    }
}

static MODULES: LazyLock<RwLock<Vec<ModuleEntry>>> =
    LazyLock::new(|| RwLock::<Vec<ModuleEntry>>::new(Vec::new()));

extern "system" fn load_module_work(
    _instance: PTP_CALLBACK_INSTANCE,
    _context: *mut core::ffi::c_void,
    _work: PTP_WORK,
) {
    if _context == (&raw const MODULES as *mut c_void) {
        log::info!("Same!");
    }
    match modules(Some(unsafe { GetCurrentProcessId() })) {
        Ok(mods) => {
            *MODULES.write() = mods;
        }
        Err(err) => {
            log::error!("{err}");
        }
    }
}

impl Default for LoadedModules {
    fn default() -> Self {
        let work = {
            unsafe {
                match CreateThreadpoolWork(
                    Some(load_module_work),
                    Some(&raw const MODULES as _),
                    None,
                ) {
                    Ok(w) => w,
                    Err(err) => {
                        log::error!("{err}");
                        Default::default()
                    }
                }
            }
        };
        Self {
            work: unsafe { Owned::new(work) },
        }
    }
}

impl Widget for &mut LoadedModules {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        unsafe {
            SubmitThreadpoolWork(*self.work);
        }
        ui.scope(|ui| {
            for entry in MODULES.read().iter() {
                ui.label(format!("{} [{}]", entry.name, entry.path));
            }
        })
        .response
    }
}
