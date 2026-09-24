#![allow(unused_imports)]

use crate::{
    config::{
        AppConfig, AssembleArgs, AssessmentPlanCliAction, AssessmentResultsCliAction,
        AttestCliAction, AttestCliArgs, AuditArgs, BlastRadiusArgs, CaptureAction, CasCliAction,
        CasCliArgs, CatalogCliAction, ClaimAction, Cli, Command, ComponentCliAction, ConvertArgs,
        DaemonCliArgs, DedupArgs, DiffArgs, EntityAction, EvidenceAction, ExportCliAction,
        ExportCliArgs, FederateCliAction, FederateCliArgs, FedrampAction, FedrampCliArgs,
        FixCliArgs, FrameworkAction, FsmCliAction, FsmCliArgs, GraphAction, GraphEdgeAction,
        GraphNodeAction, HealthArgs, ImportAction, ImportArgs, InitCliArgs, InspectArgs, LintArgs,
        MappingCliAction, ModelAction, ModelKind, OutputFormat, PipelineAction, PipelineCliArgs,
        PoamCliAction, PolicyAction, PolicyCliArgs, ProfileCliAction, ReconcileArgs, ReleaseAction,
        ResolveArgs, RulepackAction, SbomCliAction, SbomCliArgs, SearchArgs, SnapshotAction,
        SplitArgs, SspCliAction, SyncArgs, TemplateArgs, TxCliAction, TxCliArgs, ValidateArgs,
        WaiveCliAction, WaiveCliArgs, capture_root_for_args,
    },
    document::{
        BuiltinRulepack, CapsuleExporter, CasStore, ComplianceDaemon, ComplianceFederator,
        ComplianceState, ComplianceTransactionManager, DaemonConfig, DiagnosticLevel,
        EmbeddedCatalogProvider, EnterpriseCatalogBuilder, EvidenceBundle, EvidenceLevel,
        FedrampBaseline, FsmEvent, FsmRuntime, GitLabReportExporter, Jurisdiction, MergeStrategy,
        OscalDocument, PipelineConfig, PipelineOrchestrator, PolicyTarget, ReconciliationVerdict,
        SarifExporter, SbomImporter, ShieldcnBadgeConfig, ShieldcnBadgeExporter,
        SlsaProvenanceBuilder, SlsaVersion, ValidationOptions, analyze_blast_radius,
        assemble_directory, compile_policies, convert_document, deduplicate_document,
        diff_documents, ingest_policy_results, inspect_document, lint_document,
        reconcile_compliance, resolve_profile, scaffold_template, split_document, sync_and_merge,
        validate_document, validate_fedramp,
    },
    error::{AppError, Result, io_error},
    health::{HealthCheckRequest, ServingStatus},
    mcp::run_mcp_server,
    output,
    proto::oscal::services::v1::{
        ComputeTrustStateRequest, ExplainClaimRequest, ExportClaimReceiptRequest,
        GetAssessmentPlanRequest, GetAssessmentResultsRequest, GetCatalogRequest, GetClaimRequest,
        GetComponentDefinitionRequest, GetEdgeRequest, GetEntityRequest, GetEvidenceRequest,
        GetFrameworkRequest, GetMappingRequest, GetNodeRequest, GetPoamRequest, GetProfileRequest,
        GetSnapshotRequest, GetSspRequest, ImpactRadiusRequest, ListAssessmentPlansRequest,
        ListAssessmentResultsRequest, ListCatalogsRequest, ListClaimsRequest,
        ListComponentDefinitionsRequest, ListEdgesRequest, ListEntitiesRequest,
        ListFetchEventsRequest, ListFrameworksRequest, ListMappingsRequest, ListNodesRequest,
        ListPoamsRequest, ListProfilesRequest, ListProjectionEventsRequest, ListReleasesRequest,
        ListSnapshotsRequest, ListSspsRequest, ListVerificationEventsRequest, SearchRequest,
        SemanticSearchRequest, ShortestPathRequest, TraverseRequest, VerifyClosureRequest,
        VerifyEvidenceRequest,
    },
    proto::oscal::services::v1::{
        ComputeTrustStateResponse, ExplainClaimResponse, ExportClaimReceiptResponse,
        GetClaimResponse, GetEdgeResponse, GetEntityResponse, GetEvidenceResponse,
        GetFrameworkResponse, GetNodeResponse, GetSnapshotResponse, ImpactRadiusResponse,
        ListClaimsResponse, ListEdgesResponse, ListEntitiesResponse, ListFrameworksResponse,
        ListNodesResponse, ListProjectionEventsResponse, ListReleasesResponse,
        ListSnapshotsResponse, ListVerificationEventsResponse, ShortestPathResponse,
        TraverseResponse, VerifyClosureResponse, VerifyEvidenceResponse,
    },
    transport::{CrudClient, ReadOnlyClient, uuid},
};
use clap::CommandFactory;
use clap_complete::generate;
use prost::Message;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path};
use tonic::transport::Endpoint;

#[derive(Debug, Serialize)]
struct ResourceRow {
    kind: String,
    id: String,
    title: String,
    version: String,
    count: usize,
}

#[derive(Debug, Serialize)]
struct FrameworkRow {
    ref_id: String,
    name: String,
    provider: String,
    jurisdiction: String,
    version: String,
    nodes: i32,
    assessable: i32,
}

pub async fn run(config: AppConfig, command: Command) -> Result<()> {
    match command {
        Command::Capture(_)
        | Command::Proto
        | Command::Validate(_)
        | Command::Convert(_)
        | Command::Resolve(_)
        | Command::Diff(_)
        | Command::Lint(_)
        | Command::Inspect(_)
        | Command::BlastRadius(_)
        | Command::Dedup(_)
        | Command::Reconcile(_)
        | Command::Split(_)
        | Command::Assemble(_)
        | Command::Template(_)
        | Command::Policy(_)
        | Command::Fedramp(_)
        | Command::Mcp
        | Command::Sync(_)
        | Command::Audit(_)
        | Command::Fsm(_)
        | Command::Tx(_)
        | Command::Federate(_)
        | Command::Cas(_)
        | Command::Daemon(_)
        | Command::Watch(_)
        | Command::Attest(_)
        | Command::Export(_)
        | Command::Sbom(_)
        | Command::Pipeline(_)
        | Command::Fabric(_)
        | Command::Waive(_)
        | Command::Fix(_)
        | Command::Init(_)
        | Command::Gui
        | Command::Workbench => Err(AppError::Configuration(
            "local commands must be run without network configuration".to_owned(),
        )),
        Command::Completions { .. } => Err(AppError::Configuration(
            "completions must be generated without a network command".to_owned(),
        )),
        Command::Doctor => Err(AppError::Configuration(
            "doctor must be run without a network command".to_owned(),
        )),
        Command::Health(args) => health::run_health(&config, args).await,
        Command::Search(args) => search::run_search(&config, args).await,
        Command::Model(args) => model::run_model(&config, args.action).await,
        Command::Entity(args) => entity::run_entity(&config, args.action).await,
        Command::Framework(args) => framework::run_framework(&config, args.action).await,
        Command::Snapshot(args) => snapshot::run_snapshot(&config, args.action).await,
        Command::Release(args) => release::run_release(&config, args.action).await,
        Command::Import(args) => import::run_import(&config, args.action).await,
        Command::Claim(args) => match args.action {
            ClaimAction::Create { .. } | ClaimAction::Verify { .. } | ClaimAction::Sync { .. } => {
                transparency_write::run_claim(&config, args.action).await
            }
            _ => claim::run_claim(&config, args.action).await,
        },
        Command::Evidence(args) => match args.action {
            EvidenceAction::Upload { .. } => {
                transparency_write::run_evidence(&config, args.action).await
            }
            _ => evidence::run_evidence(&config, args.action).await,
        },
        Command::Graph(args) => match args.action {
            GraphAction::ProjectEdge { .. } | GraphAction::DeleteEdge { .. } => {
                graph_write::run_graph(&config, args.action).await
            }
            _ => graph::run_graph(&config, args.action).await,
        },
        Command::Catalog(args) => match args.action {
            CatalogCliAction::Crud(action) => {
                crud::run_model_crud(&config, crate::document::DocumentKind::Catalog, action).await
            }
            _ => Err(AppError::Configuration(
                "local catalog actions must be run without network configuration".to_owned(),
            )),
        },
        Command::Profile(args) => match args.action {
            ProfileCliAction::Crud(action) => {
                crud::run_model_crud(&config, crate::document::DocumentKind::Profile, action).await
            }
            _ => Err(AppError::Configuration(
                "local profile actions must be run without network configuration".to_owned(),
            )),
        },
        Command::Ssp(args) => match args.action {
            SspCliAction::Crud(action) => {
                crud::run_model_crud(&config, crate::document::DocumentKind::Ssp, action).await
            }
            _ => Err(AppError::Configuration(
                "local SSP actions must be run without network configuration".to_owned(),
            )),
        },
        Command::ComponentDefinition(args) => match args.action {
            ComponentCliAction::Crud(action) => {
                crud::run_model_crud(
                    &config,
                    crate::document::DocumentKind::ComponentDefinition,
                    action,
                )
                .await
            }
            _ => Err(AppError::Configuration(
                "local component-definition actions must be run without network configuration"
                    .to_owned(),
            )),
        },
        Command::AssessmentPlan(args) => match args.action {
            AssessmentPlanCliAction::Crud(action) => {
                crud::run_model_crud(
                    &config,
                    crate::document::DocumentKind::AssessmentPlan,
                    action,
                )
                .await
            }
            _ => Err(AppError::Configuration(
                "local assessment-plan actions must be run without network configuration"
                    .to_owned(),
            )),
        },
        Command::AssessmentResults(args) => match args.action {
            AssessmentResultsCliAction::Crud(action) => {
                crud::run_model_crud(
                    &config,
                    crate::document::DocumentKind::AssessmentResults,
                    action,
                )
                .await
            }
            _ => Err(AppError::Configuration(
                "local assessment-results actions must be run without network configuration"
                    .to_owned(),
            )),
        },
        Command::Poam(args) => match args.action {
            PoamCliAction::Crud(action) => {
                crud::run_model_crud(&config, crate::document::DocumentKind::Poam, action).await
            }
            _ => Err(AppError::Configuration(
                "local POA&M actions must be run without network configuration".to_owned(),
            )),
        },
        Command::Mapping(args) => match args.action {
            MappingCliAction::Crud(action) => {
                crud::run_model_crud(&config, crate::document::DocumentKind::Mapping, action).await
            }
            _ => Err(AppError::Configuration(
                "local mapping actions must be run without network configuration".to_owned(),
            )),
        },
        Command::Tui => Err(AppError::Configuration(
            "the TUI must be started without a second command".to_owned(),
        )),
    }
}

pub fn run_local(args: &Cli, command: Command) -> Result<()> {
    match command {
        Command::Capture(capture_args) => capture::run_capture(
            &capture_root_for_args(args),
            args.format,
            capture_args.action,
        ),
        Command::Proto => proto::run_proto(args.format),
        Command::Validate(val_args) => validate::run_validate(&val_args, args.format),
        Command::Convert(conv_args) => convert::run_convert(&conv_args),
        Command::Resolve(res_args) => resolve::run_resolve(&res_args),
        Command::Diff(diff_args) => diff::run_diff(&diff_args, args.format),
        Command::Lint(lint_args) => lint::run_lint(&lint_args, args.format),
        Command::Inspect(insp_args) => inspect::run_inspect(&insp_args, args.format),
        Command::BlastRadius(br_args) => blast_radius::run_blast_radius(&br_args, args.format),
        Command::Dedup(dedup_args) => dedup::run_dedup(&dedup_args, args.format),
        Command::Reconcile(rec_args) => reconcile::run_reconcile(&rec_args, args.format),
        Command::Split(split_args) => split::run_split(&split_args, args.format),
        Command::Assemble(assemble_args) => assemble::run_assemble(&assemble_args, args.format),
        Command::Template(tmpl_args) => template::run_template(&tmpl_args, args.format),
        Command::Policy(policy_args) => policy::run_policy(&policy_args, args.format),
        Command::Fedramp(fedramp_args) => fedramp::run_fedramp(&fedramp_args, args.format),
        Command::Mcp => run_mcp_server(),
        Command::Sync(sync_args) => sync::run_sync(&sync_args, args.format),
        Command::Audit(audit_args) => audit::run_audit(&audit_args, args.format),
        Command::Fsm(fsm_args) => fsm::run_fsm(&fsm_args, args.format),
        Command::Tx(tx_args) => tx::run_tx(&tx_args, args.format),
        Command::Federate(fed_args) => federate::run_federate(&fed_args, args.format),
        Command::Cas(cas_args) => cas::run_cas(&cas_args, args.format),
        Command::Evidence(evidence_args) => {
            evidence::run_evidence_local(&evidence_args, args.format)
        }
        Command::Daemon(d_args) | Command::Watch(d_args) => {
            daemon::run_daemon(&d_args, args.format)
        }
        Command::Attest(attest_args) => attest::run_attest(&attest_args, args.format),
        Command::Export(exp_args) => export::run_export(&exp_args, args.format),
        Command::Sbom(sbom_args) => sbom::run_sbom(&sbom_args, args.format),
        Command::Pipeline(pipe_args) => pipeline::run_pipeline(&pipe_args, args.format),
        Command::Fabric(fabric_args) => fabric::run_fabric(&fabric_args, args.format),
        Command::Waive(waive_args) => waive::run_waive(&waive_args, args.format),
        Command::Fix(fix_args) => fix::run_fix(&fix_args, args.format),
        Command::Init(init_args) => init::run_init(&init_args, args.format),
        Command::Gui | Command::Workbench => gui::run_gui(args.format),
        Command::Catalog(cat_args) => match cat_args.action {
            CatalogCliAction::Validate(v) => validate::run_validate(&v, args.format),
            CatalogCliAction::Convert(c) => convert::run_convert(&c),
            CatalogCliAction::Inspect(i) => inspect::run_inspect(&i, args.format),
            CatalogCliAction::List => catalog::run_catalog_list(args.format),
            CatalogCliAction::Crud(_) => Err(AppError::Configuration(
                "CRUD actions require network configuration".to_owned(),
            )),
            CatalogCliAction::Export {
                jurisdiction,
                output,
            } => catalog::run_catalog_export(&jurisdiction, &output, args.format),
            CatalogCliAction::ExportMatrix {
                jurisdiction,
                output,
            } => catalog::run_catalog_export_matrix(&jurisdiction, &output, args.format),
            CatalogCliAction::Extend {
                base,
                title,
                add_control_id,
                add_control_title,
                add_control_desc,
                output,
            } => catalog::run_catalog_extend(
                &base,
                &title,
                add_control_id.as_deref(),
                add_control_title.as_deref(),
                add_control_desc.as_deref(),
                &output,
                args.format,
            ),
        },
        Command::Profile(prof_args) => match prof_args.action {
            ProfileCliAction::Validate(v) => validate::run_validate(&v, args.format),
            ProfileCliAction::Convert(c) => convert::run_convert(&c),
            ProfileCliAction::Resolve(r) => resolve::run_resolve(&r),
            ProfileCliAction::Inspect(i) => inspect::run_inspect(&i, args.format),
            ProfileCliAction::Crud(_) => Err(AppError::Configuration(
                "CRUD actions require network configuration".to_owned(),
            )),
        },
        Command::Ssp(ssp_args) => match ssp_args.action {
            SspCliAction::Validate(v) => validate::run_validate(&v, args.format),
            SspCliAction::Convert(c) => convert::run_convert(&c),
            SspCliAction::Inspect(i) => inspect::run_inspect(&i, args.format),
            SspCliAction::Crud(_) => Err(AppError::Configuration(
                "CRUD actions require network configuration".to_owned(),
            )),
        },
        Command::ComponentDefinition(comp_args) => match comp_args.action {
            ComponentCliAction::Validate(v) => validate::run_validate(&v, args.format),
            ComponentCliAction::Convert(c) => convert::run_convert(&c),
            ComponentCliAction::Inspect(i) => inspect::run_inspect(&i, args.format),
            ComponentCliAction::Crud(_) => Err(AppError::Configuration(
                "CRUD actions require network configuration".to_owned(),
            )),
        },
        Command::AssessmentPlan(ap_args) => match ap_args.action {
            AssessmentPlanCliAction::Validate(v) => validate::run_validate(&v, args.format),
            AssessmentPlanCliAction::Convert(c) => convert::run_convert(&c),
            AssessmentPlanCliAction::Inspect(i) => inspect::run_inspect(&i, args.format),
            AssessmentPlanCliAction::Crud(_) => Err(AppError::Configuration(
                "CRUD actions require network configuration".to_owned(),
            )),
        },
        Command::AssessmentResults(ar_args) => match ar_args.action {
            AssessmentResultsCliAction::Validate(v) => validate::run_validate(&v, args.format),
            AssessmentResultsCliAction::Convert(c) => convert::run_convert(&c),
            AssessmentResultsCliAction::Inspect(i) => inspect::run_inspect(&i, args.format),
            AssessmentResultsCliAction::Crud(_) => Err(AppError::Configuration(
                "CRUD actions require network configuration".to_owned(),
            )),
        },
        Command::Poam(poam_args) => match poam_args.action {
            PoamCliAction::Validate(v) => validate::run_validate(&v, args.format),
            PoamCliAction::Convert(c) => convert::run_convert(&c),
            PoamCliAction::Inspect(i) => inspect::run_inspect(&i, args.format),
            PoamCliAction::Crud(_) => Err(AppError::Configuration(
                "CRUD actions require network configuration".to_owned(),
            )),
        },
        Command::Mapping(map_args) => match map_args.action {
            MappingCliAction::Validate(v) => validate::run_validate(&v, args.format),
            MappingCliAction::Convert(c) => convert::run_convert(&c),
            MappingCliAction::Inspect(i) => inspect::run_inspect(&i, args.format),
            MappingCliAction::Crud(_) => Err(AppError::Configuration(
                "CRUD actions require network configuration".to_owned(),
            )),
        },
        _ => Err(AppError::Configuration(
            "command is not local-only".to_owned(),
        )),
    }
}

pub fn run_completions(shell: clap_complete::Shell) -> Result<()> {
    let mut command = crate::config::Cli::command();
    let binary_name = command.get_name().to_owned();
    generate(shell, &mut command, binary_name, &mut std::io::stdout());
    Ok(())
}

pub fn is_local_command(command: &Command) -> bool {
    match command {
        Command::Proto
        | Command::Capture(_)
        | Command::Validate(_)
        | Command::Convert(_)
        | Command::Resolve(_)
        | Command::Diff(_)
        | Command::Lint(_)
        | Command::Inspect(_)
        | Command::BlastRadius(_)
        | Command::Dedup(_)
        | Command::Reconcile(_)
        | Command::Split(_)
        | Command::Assemble(_)
        | Command::Template(_)
        | Command::Policy(_)
        | Command::Fedramp(_)
        | Command::Mcp
        | Command::Sync(_)
        | Command::Audit(_)
        | Command::Fsm(_)
        | Command::Tx(_)
        | Command::Federate(_)
        | Command::Cas(_)
        | Command::Daemon(_)
        | Command::Watch(_)
        | Command::Attest(_)
        | Command::Export(_)
        | Command::Sbom(_)
        | Command::Pipeline(_)
        | Command::Fabric(_)
        | Command::Waive(_)
        | Command::Fix(_)
        | Command::Init(_)
        | Command::Gui
        | Command::Workbench => true,
        Command::Evidence(evidence_args) => matches!(
            evidence_args.action,
            EvidenceAction::Bundle { .. } | EvidenceAction::VerifyBundle { .. }
        ),
        Command::Catalog(cat_args) => !matches!(cat_args.action, CatalogCliAction::Crud(_)),
        Command::Profile(prof_args) => !matches!(prof_args.action, ProfileCliAction::Crud(_)),
        Command::Ssp(ssp_args) => !matches!(ssp_args.action, SspCliAction::Crud(_)),
        Command::ComponentDefinition(comp_args) => {
            !matches!(comp_args.action, ComponentCliAction::Crud(_))
        }
        Command::AssessmentPlan(ap_args) => {
            !matches!(ap_args.action, AssessmentPlanCliAction::Crud(_))
        }
        Command::AssessmentResults(ar_args) => {
            !matches!(ar_args.action, AssessmentResultsCliAction::Crud(_))
        }
        Command::Poam(poam_args) => !matches!(poam_args.action, PoamCliAction::Crud(_)),
        Command::Mapping(map_args) => !matches!(map_args.action, MappingCliAction::Crud(_)),
        Command::Import(_) => false,
        _ => false,
    }
}

pub mod assemble;
pub mod attest;
pub mod audit;
pub mod blast_radius;
pub mod capture;
pub mod cas;
pub mod catalog;
pub mod claim;
pub mod convert;
pub mod crud;
pub mod daemon;
pub mod dedup;
pub mod diff;
pub mod doctor;
pub mod entity;
pub mod evidence;
pub mod export;
pub mod fabric;
pub mod federate;
pub mod fedramp;
pub mod fix;
pub mod framework;
pub mod fsm;
pub mod graph;
pub mod graph_write;
pub mod gui;
pub mod health;
pub mod import;
pub mod init;
pub mod inspect;
pub mod lint;
pub mod model;
pub mod pipeline;
pub mod policy;
pub mod proto;
pub mod reconcile;
pub mod release;
pub mod resolve;
pub mod sbom;
pub mod search;
pub mod shared;
pub mod snapshot;
pub mod split;
pub mod sync;
pub mod template;
pub mod transparency_write;
pub mod tx;
pub mod validate;
pub mod waive;

#[allow(unused_imports)]
pub use self::shared::*;
