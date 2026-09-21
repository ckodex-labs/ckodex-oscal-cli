#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_audit(args: &AuditArgs, format: OutputFormat) -> Result<()> {
    let execute_audit = async {
        let mut auditor = crate::document::k8s::KubeAuditor::new(
            crate::document::k8s::KubeClusterClient::try_connect().await,
        );
        auditor
            .audit_cluster(
                args.namespace.as_deref(),
                args.rules_dir.as_deref(),
                args.output.as_deref(),
            )
            .await
    };

    let (_doc, report) = match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(execute_audit))?,
        Err(_) => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            rt.block_on(execute_audit)?
        }
    };

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json_str = serde_json::to_string_pretty(&report)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            println!("{json_str}");
        }
        _ => {
            println!(
                "Mizan Kubernetes Live Cluster Compliance Audit (kube-rs + Microsoft Regorus)"
            );
            println!("────────────────────────────────────────────────────────────────────────");
            println!(
                "  Cluster Connection:  {}",
                if report.cluster_connected {
                    "CONNECTED (Live K8s API)"
                } else {
                    "OFFLINE WORKLOAD SCAN (Mock Engine)"
                }
            );
            println!("  Target Namespace:    {}", report.target_namespace);
            println!("  Evaluated Pods:      {}", report.resources_evaluated);
            println!("  Satisfied Controls:  {}", report.satisfied_controls);
            println!("  Findings Generated:  {}", report.total_findings);
            if let Some(out) = &report.output_file {
                println!("  Saved Assessment:    {out}");
            }
            println!("────────────────────────────────────────────────────────────────────────");
        }
    }
    Ok(())
}
