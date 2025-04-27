use std::{fmt::Debug, ptr::NonNull};

use eframe::egui::{Widget, mutex::RwLock};
use windows::Win32::System::Threading::{
    CreateThreadpoolWork, GetCurrentProcessId, PTP_CALLBACK_INSTANCE, PTP_WORK,
    SubmitThreadpoolWork,
};
use windows_core::Owned;

use crate::utils::get_loaded_module::{ModuleEntry, modules};

pub struct LoadedModules {
    work: Owned<PTP_WORK>,
    modules: RwLock<Vec<ModuleEntry>>,
}

impl Debug for LoadedModules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedModules")
            .field("work", &self.work)
            .field("modules", &"RwLock<Vec<ModuleEntry>>")
            .finish()
    }
}

extern "system" fn load_module_work(
    _instance: PTP_CALLBACK_INSTANCE,
    context: *mut core::ffi::c_void,
    _work: PTP_WORK,
) {
    let module_rw = context.cast::<RwLock<Vec<ModuleEntry>>>();
    if let Some(module_rw) = NonNull::new(module_rw) {
        if let Ok(mods) = modules(Some(unsafe { GetCurrentProcessId() })) {
            unsafe {
                let rw = module_rw.read();
                *rw.write() = mods;
            }
        }
    }
}

impl Default for LoadedModules {
    fn default() -> Self {
        let mut modules = RwLock::new(Vec::new());
        let work = {
            unsafe {
                CreateThreadpoolWork(Some(load_module_work), Some(&raw mut modules as _), None)
                    .unwrap_or_default()
            }
        };
        Self {
            work: unsafe { Owned::new(work) },
            modules,
        }
    }
}

impl Widget for &mut LoadedModules {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        unsafe {
            SubmitThreadpoolWork(*self.work);
        }
        ui.scope(|ui| {
            for entry in self.modules.read().iter() {
                ui.label(format!("{} [{}]", entry.name, entry.path));
            }
        })
        .response
    }
}
