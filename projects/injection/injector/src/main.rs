//use std::{thread::sleep, time::Duration};

use injector::{debug_priv::enable_debug_priv, injection::Process};

fn run() -> anyhow::Result<()> {
    enable_debug_priv()?;
    let mut env_args = std::env::args();
    env_args.next();
    let target = env_args
        .next()
        .ok_or(anyhow::anyhow!("please input exe target"))?;
    let dll_path = env_args
        .next()
        .ok_or(anyhow::anyhow!("cannot find dll path"))?;
    println!("{target} / {dll_path}");
    let process = Process::open_by_exe_name(target.trim())?;
    process.inject_dll(dll_path.trim())?;
    // sleep(Duration::from_secs(40));
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
