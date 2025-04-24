use windows::{Win32::System::Threading::*, core::*};

static COUNTER: std::sync::RwLock<i32> = std::sync::RwLock::new(0);

extern "system" fn callback(_: PTP_CALLBACK_INSTANCE, _: *mut std::ffi::c_void, _: PTP_WORK) {
    let mut counter = COUNTER.write().unwrap();
    *counter += 1;
}

fn main() -> anyhow::Result<()> {
    unsafe {
        let work = CreateThreadpoolWork(Some(callback), None, None)?;
        for _ in 0..10 {
            SubmitThreadpoolWork(work);
        }
        WaitForThreadpoolWorkCallbacks(work, false);
        CloseThreadpoolWork(work);
    };
    {
        let counter = COUNTER.read().unwrap();
        println!("counter: {}", *counter);
    }
    Ok(())
}
