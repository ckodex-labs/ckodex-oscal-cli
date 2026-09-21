#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_daemon(args: &DaemonCliArgs, _format: OutputFormat) -> Result<()> {
    let config = DaemonConfig {
        watch_dir: args.dir.clone(),
        debounce_ms: args.debounce_ms,
        auto_fsm: true,
    };

    println!("Starting Mizan Continuous Compliance Daemon");
    println!("────────────────────────────────────────────────────────────────────────");
    println!("  Watching Directory:  {}", config.watch_dir.display());
    println!("  Debounce Interval:   {} ms", config.debounce_ms);
    println!("  Continuous Mode:     Active (Press Ctrl+C to stop)");
    println!("────────────────────────────────────────────────────────────────────────");

    let mut daemon = ComplianceDaemon::new(config);

    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| handle.block_on(daemon.run_continuous(args.max_cycles)))?;
    } else {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        rt.block_on(daemon.run_continuous(args.max_cycles))?;
    }

    Ok(())
}
