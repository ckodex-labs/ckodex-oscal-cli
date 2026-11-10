#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_daemon(args: &DaemonCliArgs, format: OutputFormat) -> Result<()> {
    let config = DaemonConfig {
        watch_dir: args.dir.clone(),
        debounce_ms: args.debounce_ms,
        auto_fsm: true,
    };

    let prebound_server = if let Some(ref addr_str) = args.serve {
        let addr: std::net::SocketAddr = addr_str
            .parse()
            .map_err(|e| AppError::Configuration(format!("Invalid listen address: {e}")))?;
        let std_listener = std::net::TcpListener::bind(addr).map_err(|e| {
            AppError::Configuration(format!("Failed to bind gRPC server on {addr}: {e}"))
        })?;
        std_listener.set_nonblocking(true).map_err(|e| {
            AppError::Configuration(format!(
                "Failed to configure nonblocking gRPC listener on {addr}: {e}"
            ))
        })?;
        let local_addr = std_listener.local_addr().unwrap_or(addr);
        Some((std_listener, local_addr))
    } else {
        None
    };

    let bound_addr = prebound_server.as_ref().map(|(_, addr)| *addr);
    let watch_dir_display = config.watch_dir.display().to_string();
    let mut daemon = ComplianceDaemon::new(config);
    daemon.validate_watched_file(&args.dir)?;
    let effective_debounce_ms = daemon.config().debounce_ms;

    if !matches!(format, OutputFormat::Json | OutputFormat::Jsonl) {
        println!("Starting Mizan Continuous Compliance Daemon");
        println!("────────────────────────────────────────────────────────────────────────");
        println!("  Watching Directory:  {watch_dir_display}");
        println!("  Debounce Interval:   {effective_debounce_ms} ms");
        if let Some(ref addr) = bound_addr {
            println!("  gRPC Control Plane:  http://{addr}");
        }
        println!("  Continuous Mode:     Active (Press Ctrl+C to stop)");
        println!("────────────────────────────────────────────────────────────────────────");
    }

    let max_cycles = args.max_cycles;

    let run_work = async move {
        let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let shared_shutdown_tx = std::sync::Arc::new(std::sync::Mutex::new(Some(shutdown_tx)));

        let server_handle = if let Some((std_listener, addr)) = prebound_server {
            let tokio_listener = tokio::net::TcpListener::from_std(std_listener).map_err(|e| {
                AppError::Configuration(format!(
                    "Failed to register gRPC listener on {addr} with Tokio reactor: {e}"
                ))
            })?;
            let incoming = tonic::transport::server::TcpIncoming::from(tokio_listener);
            Some(tokio::spawn(async move {
                crate::transport::start_embedded_server_with_incoming(
                    incoming,
                    crate::transport::MizanServerState::default(),
                    shutdown_rx,
                )
                .await
            }))
        } else {
            None
        };

        let stop_for_signal = stop_flag.clone();
        let shutdown_for_signal = shared_shutdown_tx.clone();
        let signal_task = tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                stop_for_signal.store(true, std::sync::atomic::Ordering::Relaxed);
                if let Some(tx) = shutdown_for_signal
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .take()
                {
                    let _ = tx.send(());
                }
            }
        });

        let daemon_result = daemon
            .run_continuous_with_shutdown(max_cycles, Some(stop_flag))
            .await;

        signal_task.abort();
        if let Some(tx) = shared_shutdown_tx
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
        {
            let _ = tx.send(());
        }
        if let Some(h) = server_handle
            && let Ok(Err(e)) = h.await
        {
            return Err(AppError::Transport(e));
        }

        daemon_result
    };

    let final_report = match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(|| handle.block_on(run_work))?
        }
        Ok(_) => std::thread::scope(|s| {
            s.spawn(|| {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| AppError::Configuration(e.to_string()))?;
                rt.block_on(run_work)
            })
            .join()
            .unwrap_or_else(|_| {
                Err(AppError::Configuration(
                    "daemon execution worker thread panicked".to_string(),
                ))
            })
        })?,
        Err(_) => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            rt.block_on(run_work)?
        }
    };

    if matches!(format, OutputFormat::Json | OutputFormat::Jsonl) {
        let payload = serde_json::json!({
            "watch_dir": watch_dir_display,
            "debounce_ms": effective_debounce_ms,
            "serve": bound_addr.map(|a| format!("http://{a}")),
            "report": final_report,
        });
        let encoded = match format {
            OutputFormat::Jsonl => serde_json::to_string(&payload)?,
            _ => serde_json::to_string_pretty(&payload)?,
        };
        println!("{encoded}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_daemon_fails_fast_on_occupied_port_and_supports_current_thread_json() {
        let occupied = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let occupied_addr = occupied.local_addr().unwrap();

        let temp_dir =
            std::env::temp_dir().join(format!("mizan-cli-daemon-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let bad_args = DaemonCliArgs {
            dir: temp_dir.clone(),
            debounce_ms: 50,
            max_cycles: Some(1),
            serve: Some(occupied_addr.to_string()),
        };
        let err = run_daemon(&bad_args, OutputFormat::Json)
            .expect_err("occupied --serve port must fail fast");
        assert!(err.to_string().contains("Failed to bind gRPC server"));

        let ok_args = DaemonCliArgs {
            dir: temp_dir.clone(),
            debounce_ms: 50,
            max_cycles: Some(1),
            serve: Some("127.0.0.1:0".to_string()),
        };
        run_daemon(&ok_args, OutputFormat::Jsonl)
            .expect("run_daemon must succeed inside CurrentThread runtime without panicking");

        drop(occupied);
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
