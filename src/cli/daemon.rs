#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_daemon(args: &DaemonCliArgs, _format: OutputFormat) -> Result<()> {
    let config = DaemonConfig {
        watch_dir: args.dir.clone(),
        debounce_ms: args.debounce_ms,
        auto_fsm: true,
    };

    let serve_addr = if let Some(ref addr_str) = args.serve {
        let addr: std::net::SocketAddr = addr_str
            .parse()
            .map_err(|e| AppError::Configuration(format!("Invalid listen address: {e}")))?;
        Some(addr)
    } else {
        None
    };

    println!("Starting Mizan Continuous Compliance Daemon");
    println!("────────────────────────────────────────────────────────────────────────");
    println!("  Watching Directory:  {}", config.watch_dir.display());
    println!("  Debounce Interval:   {} ms", config.debounce_ms);
    if let Some(ref addr) = serve_addr {
        println!("  gRPC Control Plane:  http://{}", addr);
    }
    println!("  Continuous Mode:     Active (Press Ctrl+C to stop)");
    println!("────────────────────────────────────────────────────────────────────────");

    let mut daemon = ComplianceDaemon::new(config);
    let max_cycles = args.max_cycles;

    let run_work = async move {
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let server_handle = serve_addr.map(|addr| {
            tokio::spawn(async move {
                if let Err(e) = crate::transport::start_embedded_server(addr, shutdown_rx).await {
                    eprintln!("gRPC server error: {e}");
                }
            })
        });

        let daemon_result = daemon.run_continuous(max_cycles).await;

        let _ = shutdown_tx.send(());
        if let Some(h) = server_handle {
            let _ = h.await;
        }

        daemon_result
    };

    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| handle.block_on(run_work))?;
    } else {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        rt.block_on(run_work)?;
    }

    Ok(())
}
