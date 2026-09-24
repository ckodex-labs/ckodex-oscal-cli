use serde::{Deserialize, Serialize};

use crate::document::fedramp::FedrampValidationReport;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShieldcnBadgeConfig {
    pub label: String,
    pub message: String,
    pub color: String,
    pub variant: String,
    pub wcag: Option<u8>,
    pub logo: Option<String>,
    pub provider_url: String,
}

impl Default for ShieldcnBadgeConfig {
    fn default() -> Self {
        Self {
            label: "compliance".to_string(),
            message: "verified".to_string(),
            color: "green".to_string(),
            variant: "secondary".to_string(),
            wcag: Some(3),
            logo: None,
            provider_url: "https://shieldcn.dev".to_string(),
        }
    }
}

pub struct ShieldcnBadgeExporter;

impl ShieldcnBadgeExporter {
    /// Encode a badge component string according to shields / shieldcn specification:
    /// '_' -> '__', '-' -> '--', ' ' -> '%20'
    pub fn sanitize_component(input: &str) -> String {
        input
            .replace('_', "__")
            .replace('-', "--")
            .replace(' ', "%20")
    }

    /// Build the full shieldcn-zig / shieldcn badge SVG URL.
    pub fn build_url(config: &ShieldcnBadgeConfig) -> String {
        let label_encoded = Self::sanitize_component(&config.label);
        let msg_encoded = Self::sanitize_component(&config.message);
        let color_encoded = Self::sanitize_component(&config.color);

        let mut url = format!(
            "{}/badge/{}-{}-{}.svg?variant={}",
            config.provider_url.trim_end_matches('/'),
            label_encoded,
            msg_encoded,
            color_encoded,
            config.variant
        );

        if let Some(wcag) = config.wcag {
            url.push_str(&format!("&wcag={wcag}"));
        }

        if let Some(logo) = &config.logo {
            url.push_str(&format!("&logo={logo}"));
        }

        url
    }

    /// Generate Markdown badge string.
    pub fn to_markdown(config: &ShieldcnBadgeConfig, target_link: Option<&str>) -> String {
        let svg_url = Self::build_url(config);
        let alt = format!("{}: {}", config.label, config.message);
        let link = target_link.unwrap_or("https://github.com/runbase/mizan");
        format!("[![{alt}]({svg_url})]({link})")
    }

    /// Create badge config from a FedRAMP validation report.
    pub fn from_fedramp(
        report: &FedrampValidationReport,
        variant: &str,
        wcag: Option<u8>,
        provider_url: &str,
    ) -> ShieldcnBadgeConfig {
        let is_compliant = report.is_compliant;
        let (message, color) = if is_compliant {
            (
                format!("{}·COMPLIANT", report.baseline),
                "green".to_string(),
            )
        } else {
            (
                format!("{}·{} VIOLATIONS", report.baseline, report.failed_rules),
                "red".to_string(),
            )
        };

        ShieldcnBadgeConfig {
            label: "FedRAMP".to_string(),
            message,
            color,
            variant: variant.to_string(),
            wcag,
            logo: Some("shield".to_string()),
            provider_url: provider_url.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shieldcn_badge_url_generation() {
        let cfg = ShieldcnBadgeConfig {
            label: "SLSA".to_string(),
            message: "Level 3".to_string(),
            color: "emerald".to_string(),
            variant: "secondary".to_string(),
            wcag: Some(3),
            logo: Some("shield".to_string()),
            provider_url: "https://shieldcn.dev".to_string(),
        };

        let url = ShieldcnBadgeExporter::build_url(&cfg);
        assert_eq!(
            url,
            "https://shieldcn.dev/badge/SLSA-Level%203-emerald.svg?variant=secondary&wcag=3&logo=shield"
        );

        let md = ShieldcnBadgeExporter::to_markdown(&cfg, Some("https://slsa.dev"));
        assert_eq!(
            md,
            "[![SLSA: Level 3](https://shieldcn.dev/badge/SLSA-Level%203-emerald.svg?variant=secondary&wcag=3&logo=shield)](https://slsa.dev)"
        );
    }

    #[test]
    fn test_shieldcn_from_fedramp() {
        let report = FedrampValidationReport {
            file: "catalog.json".to_string(),
            kind: "catalog".to_string(),
            baseline: "Moderate".to_string(),
            is_compliant: true,
            total_rules_checked: 85,
            passed_rules: 85,
            failed_rules: 0,
            findings: vec![],
        };

        let cfg = ShieldcnBadgeExporter::from_fedramp(
            &report,
            "secondary",
            Some(3),
            "https://shieldcn.dev",
        );
        let url = ShieldcnBadgeExporter::build_url(&cfg);
        assert!(
            url.contains("FedRAMP-Moderate%C2%B7COMPLIANT-green.svg")
                || url.contains("FedRAMP-Moderate·COMPLIANT-green.svg")
        );
        assert!(url.contains("variant=secondary"));
        assert!(url.contains("wcag=3"));
    }
}
