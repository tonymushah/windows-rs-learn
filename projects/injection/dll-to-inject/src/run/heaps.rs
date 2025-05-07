use std::{
    ffi::c_void,
    fmt::Debug,
    sync::{
        Arc, Weak,
        mpsc::{Receiver, SyncSender, sync_channel},
    },
};

use eframe::egui::{Color32, Grid, RichText, Widget, mutex::RwLock};
use windows::Win32::System::{
    Diagnostics::ToolHelp::{LF32_FIXED, LF32_FREE, LF32_MOVEABLE},
    Threading::{
        CreateThreadpoolWork, GetCurrentProcessId, PTP_CALLBACK_INSTANCE, PTP_WORK,
        SubmitThreadpoolWork,
    },
};
use windows_core::Owned;

use crate::utils::get_heap_blocks::{HeapEntry, get_heap_entries};

pub struct HeapsWidget {
    work: Owned<PTP_WORK>,
    work_context: Arc<HeapsWidgetWorkContext>,
    waker: SyncSender<()>,
}

impl Debug for HeapsWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeapsWidget")
            .field("work", &self.work)
            .field("work_context", &format!("{:p}", &self.work_context))
            .field("waker", &self.waker)
            .finish()
    }
}

struct HeapsWidgetWorkContext {
    heaps: RwLock<Vec<HeapEntry>>,
    waker: Receiver<()>,
}

extern "system" fn update_heap_work(
    _instance: PTP_CALLBACK_INSTANCE,
    _context: *mut c_void,
    _work: PTP_WORK,
) {
    let weak = if _context.is_null() {
        Weak::new()
    } else {
        unsafe { Weak::from_raw(_context.cast::<HeapsWidgetWorkContext>()) }
    };
    if let Some(context) = weak.upgrade() {
        for _ in context.waker.iter() {
            match get_heap_entries(Some(unsafe { GetCurrentProcessId() })) {
                Ok(heaps) => {
                    *context.heaps.write() = heaps;
                }
                Err(err) => {
                    log::error!("{err}");
                }
            }
        }
    }
}

impl Default for HeapsWidget {
    #[allow(clippy::arc_with_non_send_sync)]
    fn default() -> Self {
        let (tx, rx) = sync_channel::<()>(1);
        let work_context = Arc::new(HeapsWidgetWorkContext {
            heaps: RwLock::new(Vec::new()),
            waker: rx,
        });
        let work = unsafe {
            match CreateThreadpoolWork(
                Some(update_heap_work),
                Some(Arc::downgrade(&work_context).into_raw() as _),
                None,
            ) {
                Ok(w) => w,
                Err(err) => {
                    log::error!("{err}");
                    Default::default()
                }
            }
        };
        unsafe {
            SubmitThreadpoolWork(work);
        }
        Self {
            work: unsafe { Owned::new(work) },
            work_context,
            waker: tx,
        }
    }
}

impl Widget for &mut HeapsWidget {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let _ = self.waker.send(());
        ui.scope(|ui| {
            for entry in self.work_context.heaps.read().iter() {
                Grid::new(entry.heap_id).show(ui, |ui| {
                    ui.label(format!("ID: {:#x}", entry.heap_id));
                    ui.label(format!("Address: {:#x}", entry.address));
                    ui.end_row();
                    ui.label(format!("Block size: {}", entry.block_size));
                    ui.end_row();
                    match entry.flags {
                        LF32_FIXED => {
                            ui.label(
                                RichText::new("Fixed heap").color(Color32::from_rgb(200, 200, 10)),
                            );
                        }
                        LF32_FREE => {
                            ui.label(
                                RichText::new("Free heap").color(Color32::from_rgb(12, 100, 200)),
                            );
                        }
                        LF32_MOVEABLE => {
                            ui.label(
                                RichText::new("Moveable heap")
                                    .color(Color32::from_rgb(50, 255, 100)),
                            );
                        }
                        _ => {}
                    }
                    ui.end_row();
                });
            }
        })
        .response
    }
}
