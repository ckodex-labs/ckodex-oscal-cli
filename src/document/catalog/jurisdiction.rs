use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Jurisdiction {
    /// United States: NIST SP 800-53 Rev 5 & FedRAMP Rev 5 Baselines.
    UsNist800_53Rev5,
    /// Canada: Canadian Centre for Cyber Security ITSG-33 / PBMM (Protected B / Medium / Medium).
    CaItsg33Pbmm,
    /// European Union: EUCS (European Cybersecurity Scheme) & ISO/IEC 27001:2022 Controls.
    EuEucsIso27001,
    /// Enterprise Custom: Company-specific extended control catalog with overlays.
    EnterpriseCustom,
}

impl std::fmt::Display for Jurisdiction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UsNist800_53Rev5 => write!(f, "US (NIST SP 800-53 Rev 5 / FedRAMP)"),
            Self::CaItsg33Pbmm => write!(f, "Canada (CCCS ITSG-33 / PBMM)"),
            Self::EuEucsIso27001 => write!(f, "EU (EUCS / ISO/IEC 27001:2022)"),
            Self::EnterpriseCustom => write!(f, "Enterprise Custom Overlay"),
        }
    }
}

impl Jurisdiction {
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_lowercase().replace('_', "-").as_str() {
            "us" | "nist" | "fedramp" | "us-nist-800-53-rev-5" | "us-nist" => {
                Some(Self::UsNist800_53Rev5)
            }
            "ca" | "canada" | "itsg33" | "itsg-33" | "pbmm" | "ca-itsg-33-pbmm" => {
                Some(Self::CaItsg33Pbmm)
            }
            "eu" | "eucs" | "iso27001" | "iso-27001" | "eu-eucs-iso-27001" => {
                Some(Self::EuEucsIso27001)
            }
            "enterprise" | "custom" | "enterprise-custom" => Some(Self::EnterpriseCustom),
            _ => None,
        }
    }
}
