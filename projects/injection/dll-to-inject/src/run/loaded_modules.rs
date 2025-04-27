use std::{
    fmt::Debug,
    sync::{
        Arc, Weak,
        mpsc::{Receiver, SyncSender, sync_channel},
    },
};

use eframe::egui::{Widget, mutex::RwLock};
use windows::Win32::System::Threading::{
    CreateThreadpoolWork, GetCurrentProcessId, PTP_CALLBACK_INSTANCE, PTP_WORK,
    SubmitThreadpoolWork,
};
use windows_core::Owned;

use crate::utils::get_loaded_module::{ModuleEntry, modules};

struct LoadedModuleThreadContext {
    modules: RwLock<Vec<ModuleEntry>>,
    waker: Receiver<()>,
}

pub struct LoadedModules {
    work: Owned<PTP_WORK>,
    thread_context: Arc<LoadedModuleThreadContext>,
    waker: SyncSender<()>,
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
        unsafe { Weak::from_raw(_context.cast::<LoadedModuleThreadContext>()) }
    };
    if let Some(context) = weak.upgrade() {
        for _ in context.waker.iter() {
            match modules(Some(unsafe { GetCurrentProcessId() })) {
                Ok(mods) => {
                    // *MODULES.write() = mods;
                    *context.modules.write() = mods;
                }
                Err(err) => {
                    log::error!("{err}");
                }
            }
        }
    }
}

impl Default for LoadedModules {
    #[allow(clippy::arc_with_non_send_sync)]
    fn default() -> Self {
        let (tx, rx) = sync_channel(1);
        let thread_context = Arc::new(LoadedModuleThreadContext {
            modules: RwLock::new(Vec::<ModuleEntry>::new()),
            waker: rx,
        });
        let work = {
            unsafe {
                match CreateThreadpoolWork(
                    Some(load_module_work),
                    Some(Arc::downgrade(&thread_context).into_raw() as _),
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
        unsafe {
            SubmitThreadpoolWork(work);
        }
        Self {
            work: unsafe { Owned::new(work) },
            thread_context,
            waker: tx,
        }
    }
}

impl Widget for &mut LoadedModules {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let _ = self.waker.send(());
        ui.scope(|ui| {
            for entry in self.thread_context.modules.read().iter() {
                ui.label(format!("{} [{}]", entry.name, entry.path));
            }
        })
        .response
    }
}
