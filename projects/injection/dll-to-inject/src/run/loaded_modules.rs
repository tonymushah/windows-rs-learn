use std::{
    fmt::Debug,
    sync::{Arc, /*LazyLock,*/ Weak},
};

use eframe::egui::{Widget, mutex::RwLock};
use windows::Win32::System::Threading::{
    CreateThreadpoolWork, GetCurrentProcessId, PTP_CALLBACK_INSTANCE, PTP_WORK,
    SubmitThreadpoolWork,
};
use windows_core::Owned;

use crate::utils::get_loaded_module::{ModuleEntry, modules};

pub struct LoadedModules {
    work: Owned<PTP_WORK>,
    modules: Arc<RwLock<Vec<ModuleEntry>>>,
}

impl Debug for LoadedModules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedModules")
            .field("work", &self.work)
            .finish()
    }
}

/*static MODULES: LazyLock<RwLock<Vec<ModuleEntry>>> =
LazyLock::new(|| RwLock::<Vec<ModuleEntry>>::new(Vec::new()));
*/

extern "system" fn load_module_work(
    _instance: PTP_CALLBACK_INSTANCE,
    _context: *mut core::ffi::c_void,
    _work: PTP_WORK,
) {
    let weak = if _context.is_null() {
        Weak::new()
    } else {
        unsafe { Weak::from_raw(_context.cast::<RwLock<Vec<ModuleEntry>>>()) }
    };
    match modules(Some(unsafe { GetCurrentProcessId() })) {
        Ok(mods) => {
            // *MODULES.write() = mods;
            if let Some(modules) = weak.upgrade() {
                *modules.write() = mods;
            }
        }
        Err(err) => {
            log::error!("{err}");
        }
    }
}

impl Default for LoadedModules {
    #[allow(clippy::arc_with_non_send_sync)]
    fn default() -> Self {
        let modules = Arc::new(RwLock::new(Vec::<ModuleEntry>::new()));
        let work = {
            unsafe {
                match CreateThreadpoolWork(
                    Some(load_module_work),
                    Some(Arc::downgrade(&modules).into_raw() as _),
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
