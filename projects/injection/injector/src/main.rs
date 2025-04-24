use injector::injection::Process;

fn run() -> anyhow::Result<()> {
    let mut env_args = std::env::args();
    let target = env_args
        .next()
        .ok_or(anyhow::anyhow!("please input exe target"))?;
    let dll_path = env_args
        .next()
        .ok_or(anyhow::anyhow!("cannot find dll path"))?;
    let process = Process::open_by_exe_name(&target)?;
    process.inject_dll(&dll_path)?;
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
