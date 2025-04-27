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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
enum SortLabel {
    #[default]
    None,
    Name,
    Path,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
enum SortDirection {
    #[default]
    Asc,
    Desc,
}

pub struct LoadedModules {
    work: Owned<PTP_WORK>,
    thread_context: Arc<LoadedModuleThreadContext>,
    waker: SyncSender<()>,
    sort_label: SortLabel,
    sort_direction: SortDirection,
}

impl Debug for LoadedModules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedModules")
            .field("work", &self.work)
            .field("sort_label", &self.sort_label)
            .field("sort_direction", &self.sort_direction)
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
            sort_direction: Default::default(),
            sort_label: Default::default(),
        }
    }
}

impl Widget for &mut LoadedModules {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let _ = self.waker.send(());
        ui.scope(|ui| {
            ui.horizontal(|ui| {
                ui.label("Sort label:");
                ui.add_space(5.0);
                if ui
                    .selectable_value(&mut self.sort_label, SortLabel::None, "None")
                    .clicked()
                {
                    self.sort_label = SortLabel::None;
                }
                ui.add_space(2.0);
                if ui
                    .selectable_value(&mut self.sort_label, SortLabel::Name, "Name")
                    .clicked()
                {
                    self.sort_label = SortLabel::Name;
                }
                ui.add_space(2.0);
                if ui
                    .selectable_value(&mut self.sort_label, SortLabel::Path, "Path")
                    .clicked()
                {
                    self.sort_label = SortLabel::Path;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Sort direction:");
                ui.add_space(5.0);
                if ui
                    .selectable_value(&mut self.sort_direction, SortDirection::Asc, "Asc")
                    .clicked()
                {
                    self.sort_direction = SortDirection::Asc;
                }
                ui.add_space(2.0);
                if ui
                    .selectable_value(&mut self.sort_direction, SortDirection::Desc, "Desc")
                    .clicked()
                {
                    self.sort_direction = SortDirection::Desc;
                }
            });
            {
                let mut write = self.thread_context.modules.write();
                match self.sort_label {
                    SortLabel::Name => {
                        write.sort_by_cached_key(|d| d.name.clone());
                        if SortDirection::Desc == self.sort_direction {
                            write.reverse();
                        }
                    }
                    SortLabel::Path => {
                        write.sort_by_cached_key(|d| d.path.clone());
                        if SortDirection::Desc == self.sort_direction {
                            write.reverse();
                        }
                    }
                    _ => {}
                }
            }

            for entry in self.thread_context.modules.read().iter() {
                ui.label(format!("{} [{}]", entry.name, entry.path));
            }
        })
        .response
    }
}
