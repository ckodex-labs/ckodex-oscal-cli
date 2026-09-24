pub use crate::error::{AppError, Result};
pub use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
pub use std::{fs, path::PathBuf, time::Duration};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub endpoint: String,
    pub token: Option<String>,
    pub timeout: Duration,
    pub tls_domain: Option<String>,
    pub ca_cert: Option<PathBuf>,
    pub client_cert: Option<PathBuf>,
    pub client_key: Option<PathBuf>,
    pub output: OutputFormat,
    pub theme: Theme,
    pub capture_root: PathBuf,
    pub capture_enabled: bool,
    pub read_only: bool,
    pub valence: bool,
}

impl AppConfig {
    pub fn from_args(args: &Cli) -> Result<Self> {
        let endpoint = normalized_endpoint(&args.endpoint)?;
        if args.timeout_secs == 0 {
            return Err(AppError::Configuration(
                "timeout-secs must be greater than zero".to_owned(),
            ));
        }
        if args.client_cert.is_some() != args.client_key.is_some() {
            return Err(AppError::Configuration(
                "client-cert and client-key must be supplied together".to_owned(),
            ));
        }
        if args.token.is_some() && args.token_file.is_some() {
            return Err(AppError::Configuration(
                "token and token-file must not be supplied together".to_owned(),
            ));
        }

        Ok(Self {
            endpoint,
            token: match (&args.token, &args.token_file) {
                (Some(token), None) => Some(normalize_token(token)?),
                (None, Some(path)) => Some(normalize_token(
                    &fs::read_to_string(path)
                        .map_err(|error| crate::error::io_error(path, error))?,
                )?),
                (None, None) => None,
                (Some(_), Some(_)) => {
                    return Err(AppError::Configuration(
                        "token and token-file must not be supplied together".to_owned(),
                    ));
                }
            },
            timeout: Duration::from_secs(args.timeout_secs),
            tls_domain: args.tls_domain.clone(),
            ca_cert: args.ca_cert.clone(),
            client_cert: args.client_cert.clone(),
            client_key: args.client_key.clone(),
            output: args.format,
            theme: args.theme,
            capture_root: capture_root_for_args(args),
            capture_enabled: args.capture,
            read_only: args.read_only,
            valence: args.valence,
        })
    }
}

pub(crate) fn normalized_endpoint(raw_endpoint: &str) -> Result<String> {
    let raw_endpoint = raw_endpoint.trim();
    if raw_endpoint.is_empty() {
        return Err(AppError::Configuration(
            "endpoint must not be empty".to_owned(),
        ));
    }
    Ok(if raw_endpoint.contains("://") {
        raw_endpoint.to_owned()
    } else {
        format!("http://{raw_endpoint}")
    })
}

pub(crate) fn capture_root_for_args(args: &Cli) -> PathBuf {
    args.capture_dir
        .clone()
        .unwrap_or_else(default_capture_root)
}

fn default_capture_root() -> PathBuf {
    directories::ProjectDirs::from("ca", "Cortaix", "OSCAL CLI")
        .map(|dirs| dirs.data_local_dir().join("captures"))
        .unwrap_or_else(|| PathBuf::from(".oscal-cli/captures"))
}

fn normalize_token(token: &str) -> Result<String> {
    let token = token.trim();
    if token.is_empty() {
        return Err(AppError::Configuration(
            "token must not be empty".to_owned(),
        ));
    }
    Ok(token.to_owned())
}

#[derive(Clone, Debug, Parser)]
#[command(
    name = "oscal-cli",
    version,
    about = "CLI and terminal observer for OSCALify protobuf services"
)]
pub struct Cli {
    /// gRPC endpoint to observe; a scheme is optional and defaults to http.
    #[arg(
        long,
        global = true,
        env = "OSCALIFY_ENDPOINT",
        default_value = "127.0.0.1:50051"
    )]
    pub endpoint: String,
    /// Bearer token sent only as authorization metadata; never captured.
    #[arg(long, global = true, env = "OSCALIFY_TOKEN", hide_env_values = true)]
    pub token: Option<String>,
    /// File containing a trimmed bearer token; mutually exclusive with --token.
    #[arg(long, global = true, value_name = "FILE", conflicts_with = "token")]
    pub token_file: Option<PathBuf>,
    /// TLS server-name override used for certificate verification.
    #[arg(long, global = true, value_name = "DOMAIN")]
    pub tls_domain: Option<String>,
    /// PEM CA bundle used in addition to the platform trust store.
    #[arg(long, global = true, value_name = "PEM")]
    pub ca_cert: Option<PathBuf>,
    /// PEM client certificate for mutual TLS.
    #[arg(long, global = true, value_name = "PEM")]
    pub client_cert: Option<PathBuf>,
    /// PEM private key paired with --client-cert for mutual TLS.
    #[arg(long, global = true, value_name = "PEM")]
    pub client_key: Option<PathBuf>,
    /// Network timeout in seconds.
    #[arg(long, global = true, default_value_t = 10)]
    pub timeout_secs: u64,
    /// Human or machine-readable output format.
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
    /// TUI visual theme; ignored by non-interactive commands.
    #[arg(long, global = true, value_enum, default_value_t = Theme::Ledger)]
    pub theme: Theme,
    /// Local root for verified request/response captures.
    #[arg(long, global = true, value_name = "DIR")]
    pub capture_dir: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help = "Persist request/response protobuf captures locally"
    )]
    pub capture: bool,
    /// Deny mutating RPCs before they are sent to the server.
    #[arg(long, global = true, default_value_t = false)]
    pub read_only: bool,
    /// Include valence labels in CRUD output.
    #[arg(long, global = true, default_value_t = false)]
    pub valence: bool,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Open the interactive observer.
    Tui,
    /// Search OSCAL models or governance entities.
    Search(SearchArgs),
    /// Browse OSCAL model documents.
    Model(ModelArgs),
    /// Browse governance entities.
    Entity(EntityArgs),
    /// Browse governance framework summaries.
    Framework(FrameworkArgs),
    /// Browse governance snapshots.
    Snapshot(SnapshotArgs),
    /// Browse governance releases.
    Release(ReleaseArgs),
    /// Inspect transparency claims.
    Claim(ClaimArgs),
    /// Inspect content-addressed evidence.
    Evidence(EvidenceArgs),
    /// Explore the transparency graph and its proof state.
    Graph(GraphArgs),
    /// Import transparency claims and evidence in batches.
    Import(ImportArgs),
    /// Inspect local read-only captures.
    Capture(CaptureArgs),
    /// Inspect the protocol binding snapshot.
    Proto,
    /// Generate shell completion scripts.
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Inspect local configuration without dialing a server.
    Doctor,
    /// Check standard gRPC service health.
    Health(HealthArgs),
    /// Validate an OSCAL document against official schemas and constraints.
    Validate(ValidateArgs),
    /// Convert an OSCAL document between JSON and YAML formats.
    Convert(ConvertArgs),
    /// Resolve an OSCAL profile into an effective catalog.
    Resolve(ResolveArgs),
    /// Semantically diff two OSCAL documents.
    Diff(DiffArgs),
    /// Lint and optionally autofix common OSCAL document issues.
    Lint(LintArgs),
    /// Inspect and summarize an OSCAL document structure.
    Inspect(InspectArgs),
    /// Operations on OSCAL catalogs.
    Catalog(CatalogCliArgs),
    /// Operations on OSCAL profiles.
    Profile(ProfileCliArgs),
    /// Operations on OSCAL system security plans.
    Ssp(SspCliArgs),
    /// Operations on OSCAL component definitions.
    ComponentDefinition(ComponentCliArgs),
    /// Operations on OSCAL assessment plans.
    AssessmentPlan(AssessmentPlanCliArgs),
    /// Operations on OSCAL assessment results.
    AssessmentResults(AssessmentResultsCliArgs),
    /// Operations on OSCAL plans of action and milestones.
    Poam(PoamCliArgs),
    /// Operations on OSCAL mapping collections.
    Mapping(MappingCliArgs),
    /// Analyze the multi-model compliance blast radius for a control, component, or parameter.
    BlastRadius(BlastRadiusArgs),
    /// Detect and eliminate duplicate controls, components, parties, and resources.
    Dedup(DedupArgs),
    /// Reconcile declared compliance against observed inventory and assessment findings.
    Reconcile(ReconcileArgs),
    /// Split an OSCAL document into an agile Markdown authoring workspace.
    Split(SplitArgs),
    /// Assemble an agile Markdown workspace into a validated OSCAL document.
    Assemble(AssembleArgs),
    /// Scaffold a new standard OSCAL artifact (NIST 800-53, FedRAMP, ISO 27001, SOC 2).
    Template(TemplateArgs),
    /// Compile OSCAL to OPA/Rego or Kyverno policies, or ingest policy evaluation logs.
    Policy(PolicyCliArgs),
    /// Validate an SSP or Component Definition against FedRAMP PMO baseline rules.
    Fedramp(FedrampCliArgs),
    /// Run the native AI Model Context Protocol (MCP) server over stdio.
    Mcp,
    /// Perform a 3-way GitOps AST merge across base, upstream, and local documents.
    Sync(SyncArgs),
    /// Audit a live Kubernetes cluster or workload manifests using in-process Regorus policies.
    Audit(AuditArgs),
    /// Manage the compliance document Finite State Machine (FSM) lifecycle.
    Fsm(FsmCliArgs),
    /// Execute ACID compliance transactions with Write-Ahead Logging (WAL) and rollback.
    Tx(TxCliArgs),
    /// Cross-link and federate Catalogs, Profiles, SSPs, Live Audits, and POA&Ms.
    Federate(FederateCliArgs),
    /// Manage Content-Addressable Storage (CAS) objects and evidence caches.
    Cas(CasCliArgs),
    /// Run the continuous compliance background daemon.
    Daemon(DaemonCliArgs),
    /// Watch a workspace directory and continuously reconcile OSCAL documents.
    Watch(DaemonCliArgs),
    /// Generate and verify SLSA v1.2 / v1.0 and in-toto supply-chain attestations.
    Attest(AttestCliArgs),
    /// Export OSCAL documents to standard CI/CD formats (SARIF v2.1.0, GitLab Security Scanner).
    Export(ExportCliArgs),
    /// Ingest and correlate Software Bill of Materials (SBOM) with OSCAL components.
    Sbom(SbomCliArgs),
    /// Execute an end-to-end zero-trust compliance pipeline in a single step.
    Pipeline(PipelineCliArgs),
    /// Enterprise Root Fabric: Multi-tenancy, SPIFFE/SPIRE identity, OIDC & partitioned datastore.
    Fabric(FabricCliArgs),
    /// Create, list, or revoke time-bounded cryptographic derogation waivers.
    Waive(WaiveCliArgs),
    /// Automatically remediate and patch policy violations in Kubernetes manifests and Dockerfiles.
    Fix(FixCliArgs),
    /// Auto-discover compliance controls from codebases (Dockerfiles, k8s, lockfiles) and scaffold governance.
    Init(InitCliArgs),
    /// Launch the Mizan interactive desktop workbench (Tauri / Browser GUI).
    Gui,
    /// Launch the Mizan interactive desktop workbench (Tauri / Browser GUI).
    Workbench,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Jsonl,
    Proto,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Theme {
    Ledger,
    Vault,
    Hc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum TargetFormat {
    Json,
    Yaml,
    Csv,
    Proto,
}

impl TargetFormat {
    pub fn to_file_format(self) -> crate::document::FileFormat {
        match self {
            Self::Json => crate::document::FileFormat::Json,
            Self::Yaml => crate::document::FileFormat::Yaml,
            Self::Csv => crate::document::FileFormat::Csv,
            Self::Proto => crate::document::FileFormat::Proto,
        }
    }
}

pub mod assemble;
pub mod assessmentplancli;
pub mod assessmentresultscli;
pub mod attestcli;
pub mod audit;
pub mod blastradius;
pub mod capture;
pub mod cascli;
pub mod catalogcli;
pub mod claim;
pub mod componentcli;
pub mod convert;
pub mod crud;
pub mod daemoncli;
pub mod dedup;
pub mod diff;
pub mod entity;
pub mod evidence;
pub mod exportcli;
pub mod fabric;
pub mod fabriccli;
pub mod fabricdatastore;
pub mod fabrictenant;
pub mod federatecli;
pub mod fedramp;
pub mod fedrampcli;
pub mod fixcli;
pub mod framework;
pub mod fsmcli;
pub mod graph;
pub mod graphedge;
pub mod graphnode;
pub mod health;
pub mod import;
pub mod initcli;
pub mod inspect;
pub mod lint;
pub mod mappingcli;
pub mod model;
pub mod pipeline;
pub mod pipelinecli;
pub mod poamcli;
pub mod policy;
pub mod policycli;
pub mod profilecli;
pub mod reconcile;
pub mod release;
pub mod resolve;
pub mod sbomcli;
pub mod search;
pub mod snapshot;
pub mod split;
pub mod sspcli;
pub mod sync;
pub mod template;
pub mod txcli;
pub mod validate;
pub mod waivecli;

pub use self::assemble::AssembleArgs;
pub use self::assessmentplancli::AssessmentPlanCliAction;
pub use self::assessmentplancli::AssessmentPlanCliArgs;
pub use self::assessmentresultscli::AssessmentResultsCliAction;
pub use self::assessmentresultscli::AssessmentResultsCliArgs;
pub use self::attestcli::AttestCliAction;
pub use self::attestcli::AttestCliArgs;
pub use self::audit::AuditArgs;
pub use self::blastradius::BlastRadiusArgs;
pub use self::capture::CaptureAction;
pub use self::capture::CaptureArgs;
pub use self::cascli::CasCliAction;
pub use self::cascli::CasCliArgs;
pub use self::catalogcli::CatalogCliAction;
pub use self::catalogcli::CatalogCliArgs;
pub use self::claim::ClaimAction;
pub use self::claim::ClaimArgs;
pub use self::componentcli::ComponentCliAction;
pub use self::componentcli::ComponentCliArgs;
pub use self::convert::ConvertArgs;
pub use self::crud::CrudAction;
pub use self::daemoncli::DaemonCliArgs;
pub use self::dedup::DedupArgs;
pub use self::diff::DiffArgs;
pub use self::entity::EntityAction;
pub use self::entity::EntityArgs;
pub use self::evidence::EvidenceAction;
pub use self::evidence::EvidenceArgs;
pub use self::exportcli::ExportCliAction;
pub use self::exportcli::ExportCliArgs;
pub use self::fabric::FabricAction;
pub use self::fabriccli::FabricCliArgs;
pub use self::fabricdatastore::FabricDatastoreAction;
pub use self::fabrictenant::FabricTenantAction;
pub use self::federatecli::FederateCliAction;
pub use self::federatecli::FederateCliArgs;
pub use self::fedramp::FedrampAction;
pub use self::fedrampcli::FedrampCliArgs;
pub use self::fixcli::FixCliArgs;
pub use self::framework::FrameworkAction;
pub use self::framework::FrameworkArgs;
pub use self::fsmcli::FsmCliAction;
pub use self::fsmcli::FsmCliArgs;
pub use self::graph::GraphAction;
pub use self::graph::GraphArgs;
pub use self::graphedge::GraphEdgeAction;
pub use self::graphnode::GraphNodeAction;
pub use self::health::HealthArgs;
pub use self::import::ImportAction;
pub use self::import::ImportArgs;
pub use self::initcli::InitCliArgs;
pub use self::inspect::InspectArgs;
pub use self::lint::LintArgs;
pub use self::mappingcli::MappingCliAction;
pub use self::mappingcli::MappingCliArgs;
pub use self::model::ModelAction;
pub use self::model::ModelArgs;
pub use self::model::ModelKind;
pub use self::pipeline::PipelineAction;
pub use self::pipelinecli::PipelineCliArgs;
pub use self::poamcli::PoamCliAction;
pub use self::poamcli::PoamCliArgs;
pub use self::policy::PolicyAction;
pub use self::policy::RulepackAction;
pub use self::policycli::PolicyCliArgs;
pub use self::profilecli::ProfileCliAction;
pub use self::profilecli::ProfileCliArgs;
pub use self::reconcile::ReconcileArgs;
pub use self::release::ReleaseAction;
pub use self::release::ReleaseArgs;
pub use self::resolve::ResolveArgs;
pub use self::sbomcli::SbomCliAction;
pub use self::sbomcli::SbomCliArgs;
pub use self::search::SearchArgs;
pub use self::snapshot::SnapshotAction;
pub use self::snapshot::SnapshotArgs;
pub use self::split::SplitArgs;
pub use self::sspcli::SspCliAction;
pub use self::sspcli::SspCliArgs;
pub use self::sync::SyncArgs;
pub use self::template::TemplateArgs;
pub use self::txcli::TxCliAction;
pub use self::txcli::TxCliArgs;
pub use self::validate::ValidateArgs;
pub use self::waivecli::{WaiveCliAction, WaiveCliArgs};

#[cfg(test)]
mod tests;
