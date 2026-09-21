#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, Subcommand)]
pub enum PolicyAction {
    /// Compile an OSCAL Component Definition or SSP into OPA/Rego or Kyverno policies.
    Compile {
        /// Path to source OSCAL document.
        file: PathBuf,
        /// Target policy engine (rego, kyverno, all).
        #[arg(long, default_value = "rego")]
        target: String,
        /// Output directory for compiled policy files.
        #[arg(long, short = 'o')]
        output_dir: PathBuf,
    },
    /// Ingest OPA / Kyverno evaluation audit logs into an OSCAL assessment-results document.
    Ingest {
        /// Path to policy evaluation audit log JSON file.
        log_file: PathBuf,
        /// Title for the assessment results document.
        #[arg(long, default_value = "Continuous Policy Assessment")]
        title: String,
        /// Output file path for the generated assessment-results document.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
    /// Evaluate an input JSON/YAML artifact against compiled OSCAL Rego rules using Microsoft Regorus.
    Eval {
        /// Path to compiled Rego policy file or directory.
        #[arg(long, short = 'p')]
        policy: PathBuf,
        /// Path to input JSON or YAML file to evaluate.
        #[arg(long, short = 'i')]
        input: PathBuf,
        /// Rego package or query to evaluate (e.g. data.oscal.k8s or oscal.ac_2).
        #[arg(long, default_value = "data")]
        query: String,
    },
    /// Manage built-in Rego compliance rulepacks (CIS K8s Benchmarks, FedRAMP, ITSG-33).
    Rulepack {
        #[command(subcommand)]
        action: RulepackAction,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum RulepackAction {
    /// List all embedded Rego compliance rulepack policies.
    List,
    /// Evaluate a built-in compliance rule against an input workload JSON/YAML.
    Eval {
        /// Built-in rule ID (e.g. cis-k8s-5.2.1, fedramp-ac-2, itsg33-boundary-isolation).
        #[arg(long, short = 'r')]
        rule: String,
        /// Path to input JSON or YAML workload file.
        #[arg(long, short = 'i')]
        input: PathBuf,
    },
}
