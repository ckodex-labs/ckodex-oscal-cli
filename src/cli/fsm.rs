#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_fsm(args: &FsmCliArgs, format: OutputFormat) -> Result<()> {
    match &args.action {
        FsmCliAction::Status { state, evidence } => {
            let st = match state.to_lowercase().as_str() {
                "under_review" => ComplianceState::UnderReview,
                "approved" => ComplianceState::Approved,
                "operational" => ComplianceState::Operational,
                "degraded" => ComplianceState::Degraded,
                "remediating" => ComplianceState::Remediating,
                "archived" => ComplianceState::Archived,
                _ => ComplianceState::Draft,
            };
            let ev = match evidence.to_lowercase().as_str() {
                "e1" => EvidenceLevel::E1SchemaValid,
                "e2" => EvidenceLevel::E2LinterPassed,
                "e3" => EvidenceLevel::E3FedrampPassed,
                "e4" => EvidenceLevel::E4AuditPassed,
                "e5" => EvidenceLevel::E5ContinuousMonitoring,
                _ => EvidenceLevel::E0Unverified,
            };

            let runtime = FsmRuntime::new(st, ev);
            let history = runtime.history();

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(history)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan Compliance Lifecycle Finite State Machine (FSM)");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Current State:       {}", runtime.state());
                    println!("  Evidence Level:      {}", runtime.evidence());
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
        FsmCliAction::Transition {
            from,
            evidence,
            event,
            actor,
            rationale,
        } => {
            let st = match from.to_lowercase().as_str() {
                "under_review" => ComplianceState::UnderReview,
                "approved" => ComplianceState::Approved,
                "operational" => ComplianceState::Operational,
                "degraded" => ComplianceState::Degraded,
                "remediating" => ComplianceState::Remediating,
                "archived" => ComplianceState::Archived,
                _ => ComplianceState::Draft,
            };
            let ev = match evidence.to_lowercase().as_str() {
                "e1" => EvidenceLevel::E1SchemaValid,
                "e2" => EvidenceLevel::E2LinterPassed,
                "e3" => EvidenceLevel::E3FedrampPassed,
                "e4" => EvidenceLevel::E4AuditPassed,
                "e5" => EvidenceLevel::E5ContinuousMonitoring,
                _ => EvidenceLevel::E0Unverified,
            };

            let ev_type = match event.to_lowercase().as_str() {
                "submit_for_review" => FsmEvent::SubmitForReview,
                "approve" => FsmEvent::Approve,
                "deploy_to_cluster" => FsmEvent::DeployToCluster,
                "resolve_remediation" => FsmEvent::ResolveRemediation,
                "archive" => FsmEvent::Archive,
                _ => {
                    return Err(AppError::Configuration(format!(
                        "Unknown FSM event: {event} (use submit_for_review, approve, deploy_to_cluster, resolve_remediation, archive)"
                    )));
                }
            };

            let mut runtime = FsmRuntime::new(st, ev);
            let next_state = runtime.transition(ev_type, actor, rationale)?;

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(runtime.history())
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan FSM State Transition Verified & Recorded");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Origin State:        {from}");
                    println!("  Trigger Event:       {event}");
                    println!("  Next State:          {next_state}");
                    println!("  Actor:               {actor}");
                    println!("  Rationale:           {rationale}");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
    }
    Ok(())
}
