#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_catalog_list(format: OutputFormat) -> Result<()> {
    let list = serde_json::json!([
        {
            "code": "us",
            "jurisdiction": "United States",
            "standard": "NIST SP 800-53 Rev 5 / FedRAMP Rev 5 Core Baseline",
            "controls_count": 9
        },
        {
            "code": "ca",
            "jurisdiction": "Canada",
            "standard": "CCCS ITSG-33 / Protected B Medium Medium (PBMM)",
            "controls_count": 4
        },
        {
            "code": "eu",
            "jurisdiction": "European Union",
            "standard": "EUCS / ISO/IEC 27001:2022 Controls Mapping",
            "controls_count": 4
        },
        {
            "code": "enterprise",
            "jurisdiction": "Enterprise",
            "standard": "Enterprise Custom Overlay Baseline Template",
            "controls_count": 1
        }
    ]);

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json_str = serde_json::to_string_pretty(&list)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            println!("{json_str}");
        }
        _ => {
            println!("Mizan Built-in Tri-Jurisdictional Baseline Catalogs");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  [us]         US NIST SP 800-53 Rev 5 / FedRAMP Rev 5 Baseline");
            println!("  [ca]         Canada CCCS ITSG-33 Protected B Medium Medium (PBMM)");
            println!("  [eu]         EU Cybersecurity Scheme (EUCS) & ISO/IEC 27001:2022");
            println!("  [enterprise] Enterprise Custom Baseline Overlay");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  Use `mizan catalog export -j <us|ca|eu|enterprise> -o catalog.json`");
        }
    }
    Ok(())
}

pub(super) fn run_catalog_export(
    jurisdiction_str: &str,
    output: &std::path::Path,
    format: OutputFormat,
) -> Result<()> {
    let jur = Jurisdiction::from_str_name(jurisdiction_str).ok_or_else(|| {
        AppError::Configuration(format!(
            "Unknown jurisdiction: {jurisdiction_str} (use us, ca, eu, enterprise)"
        ))
    })?;

    let doc = EmbeddedCatalogProvider::get_catalog(jur)?;
    if let Some(parent) = output.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json_str = serde_json::to_string_pretty(&doc.value)
        .map_err(|e| AppError::Configuration(e.to_string()))?;
    std::fs::write(output, json_str).map_err(|e| io_error(output, e))?;

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let rep = serde_json::json!({
                "jurisdiction": jur.to_string(),
                "output_file": output.display().to_string(),
                "status": "exported"
            });
            println!("{rep}");
        }
        _ => {
            println!("Exported {jur} baseline catalog -> {}", output.display());
        }
    }
    Ok(())
}

pub(super) fn run_catalog_extend(
    base_str: &str,
    title: &str,
    add_control_id: Option<&str>,
    add_control_title: Option<&str>,
    add_control_desc: Option<&str>,
    output: &std::path::Path,
    format: OutputFormat,
) -> Result<()> {
    let base = Jurisdiction::from_str_name(base_str).ok_or_else(|| {
        AppError::Configuration(format!(
            "Unknown base jurisdiction: {base_str} (use us, ca, eu)"
        ))
    })?;

    let mut builder = EnterpriseCatalogBuilder::new(title).with_base_jurisdiction(base);

    if let (Some(cid), Some(ctitle), Some(cdesc)) =
        (add_control_id, add_control_title, add_control_desc)
    {
        builder = builder.add_custom_control(cid, ctitle, cdesc, "Enterprise-Policy");
    }

    let doc = builder.build(Some(output))?;
    let root = doc.root_object().unwrap();
    let ctrls_len = root
        .get("controls")
        .and_then(serde_json::Value::as_array)
        .map_or(0, |a| a.len());

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let rep = serde_json::json!({
                "title": title,
                "base_jurisdiction": base.to_string(),
                "total_controls": ctrls_len,
                "output_file": output.display().to_string(),
            });
            println!("{rep}");
        }
        _ => {
            println!("Created Enterprise Catalog: {title}");
            println!("  Base Jurisdiction: {}", base);
            println!("  Total Controls:    {}", ctrls_len);
            println!("  Saved To:          {}", output.display());
        }
    }
    Ok(())
}
