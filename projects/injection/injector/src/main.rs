//use std::{thread::sleep, time::Duration};

use std::process::Command;

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
    let mut sp_proc = Command::new(target).spawn()?;

    let process = Process::open(sp_proc.id())?;
    process.inject_dll(dll_path.trim())?;
    sp_proc.wait()?;
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
