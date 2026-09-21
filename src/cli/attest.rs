#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_attest(args: &AttestCliArgs, format: OutputFormat) -> Result<()> {
    match &args.action {
        AttestCliAction::Slsa {
            subject,
            digest,
            version,
            evidence,
            output,
        } => {
            let slsa_ver = SlsaVersion::from_str_name(version).ok_or_else(|| {
                AppError::Configuration(format!(
                    "Unknown SLSA version: {version} (use v1.2 or v1.0)"
                ))
            })?;

            let mut builder = SlsaProvenanceBuilder::new(subject, digest).with_version(slsa_ver);

            if let Some(ev_path) = evidence {
                let bundle = EvidenceBundle::load_from_file(ev_path)?;
                builder = builder.with_oscal_evidence(bundle);
            }

            let statement = builder.build();

            if let Some(out_p) = output {
                SlsaProvenanceBuilder::save_to_file(&statement, out_p)?;
            }

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(&statement)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan SLSA Supply Chain Provenance Attestation Statement");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Specification:       {}", slsa_ver.predicate_type());
                    println!("  Subject:             {subject}");
                    println!("  Subject Digest:      {digest}");
                    if let Some(out) = output {
                        println!("  Saved Statement:     {}", out.display());
                    }
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
        AttestCliAction::Verify { statement_file } => {
            let content =
                std::fs::read_to_string(statement_file).map_err(|e| io_error(statement_file, e))?;
            let statement: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| AppError::Configuration(e.to_string()))?;

            let report = SlsaProvenanceBuilder::verify(&statement)?;

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(&report)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan SLSA Provenance Verification");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Subject:             {}", report.subject_name);
                    println!("  Subject SHA-256:     {}", report.subject_sha256);
                    println!("  Builder ID:          {}", report.builder_id);
                    println!(
                        "  Predicate:           {}",
                        report.slsa_version.predicate_type()
                    );
                    println!(
                        "  OSCAL Proof Linked:  {}",
                        if report.has_oscal_evidence {
                            "YES"
                        } else {
                            "NO"
                        }
                    );
                    if let Some(root) = &report.merkle_root {
                        println!("  Evidence Merkle Root: {root}");
                    }
                    println!("  Verdict:             VALID");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
    }
    Ok(())
}
