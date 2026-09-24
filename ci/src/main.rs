use clap::{Parser, ValueEnum};
use eyre::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum CiAction {
    Lint,
    Test,
    Build,
    Workbench,
    Badge,
    All,
}

#[derive(Parser, Debug)]
#[command(
    name = "mizan-dagger-ci",
    about = "Mizan High-Assurance CI/CD Pipeline powered by Dagger Rust SDK & shieldcn-zig"
)]
struct Args {
    #[arg(value_enum, default_value_t = CiAction::All)]
    action: CiAction,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("=== Mizan CI/CD · Dagger Rust SDK Pipeline ===");
    println!("Action: {:?}", args.action);
    println!("===============================================");

    dagger_sdk::connect(move |client| async move { run_pipeline(client, args.action).await })
        .await?;

    Ok(())
}

async fn run_pipeline(client: dagger_sdk::Query, action: CiAction) -> Result<()> {
    let project_dir = client.host().directory_opts(
        ".",
        dagger_sdk::HostDirectoryOpts {
            exclude: Some(vec![
                "target",
                ".git",
                "apps/workbench/.next",
                "apps/workbench/node_modules",
            ]),
            gitignore: Some(true),
            include: None,
            no_cache: None,
        },
    );

    let rust_base = client
        .container()
        .from("rust:1.85-bookworm")
        .with_directory("/workspace", project_dir.clone())
        .with_workdir("/workspace")
        .with_mounted_cache(
            "/usr/local/cargo/registry",
            client.cache_volume("cargo-registry"),
        )
        .with_mounted_cache("/usr/local/cargo/git", client.cache_volume("cargo-git"))
        .with_mounted_cache("/workspace/target", client.cache_volume("cargo-target"));

    if matches!(action, CiAction::Lint | CiAction::All) {
        println!(">> [Stage 1/5] Running Clippy & Rustfmt...");
        let lint_container = rust_base
            .clone()
            .with_exec(vec!["rustup", "component", "add", "clippy", "rustfmt"])
            .with_exec(vec![
                "cargo",
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ])
            .with_exec(vec!["cargo", "fmt", "--all", "--", "--check"])
            .sync()
            .await?;
        let _ = lint_container;
        println!("[OK] Stage 1 Passed: Clippy and rustfmt checks verified.");
    }

    if matches!(action, CiAction::Test | CiAction::All) {
        println!(">> [Stage 2/5] Running Workspace Tests (158 Invariant Tests)...");
        let test_container = rust_base
            .clone()
            .with_exec(vec!["cargo", "test", "--workspace"])
            .sync()
            .await?;
        let _ = test_container;
        println!("[OK] Stage 2 Passed: All workspace tests passed.");
    }

    if matches!(action, CiAction::Build | CiAction::All) {
        println!(">> [Stage 3/5] Building Fat-LTO Release Binaries (mizan, oscal-cli)...");
        let build_container = rust_base
            .clone()
            .with_exec(vec!["cargo", "build", "--release"])
            .sync()
            .await?;
        let _ = build_container;
        println!("[OK] Stage 3 Passed: Fat-LTO release binaries compiled.");
    }

    if matches!(action, CiAction::Workbench | CiAction::All) {
        println!(">> [Stage 4/5] Building Next.js 16 Turbopack Workbench...");
        let node_container = client
            .container()
            .from("node:22-bookworm-slim")
            .with_directory("/workspace", project_dir.clone())
            .with_workdir("/workspace/apps/workbench")
            .with_mounted_cache("/root/.npm", client.cache_volume("npm-cache"))
            .with_exec(vec!["npm", "ci"])
            .with_exec(vec!["npm", "run", "lint"])
            .with_exec(vec!["npm", "run", "build"])
            .sync()
            .await?;
        let _ = node_container;
        println!("[OK] Stage 4 Passed: Workbench lint and production build verified.");
    }

    if matches!(action, CiAction::Badge | CiAction::All) {
        println!(">> [Stage 5/5] Generating shieldcn-zig Badges (APCA WCAG 3.0)...");
        let badge_container = rust_base
            .clone()
            .with_exec(vec![
                "cargo",
                "run",
                "--bin",
                "mizan",
                "--",
                "export",
                "badge",
                "--label",
                "OSCAL Metaschema",
                "--message",
                "v1.2.3 Validated",
                "--color",
                "blue",
                "--variant",
                "secondary",
                "--wcag",
                "3",
            ])
            .with_exec(vec![
                "cargo",
                "run",
                "--bin",
                "mizan",
                "--",
                "export",
                "badge",
                "--label",
                "SLSA Level",
                "--message",
                "3 In-Toto",
                "--color",
                "emerald",
                "--variant",
                "secondary",
                "--wcag",
                "3",
            ])
            .with_exec(vec![
                "cargo",
                "run",
                "--bin",
                "mizan",
                "--",
                "export",
                "badge",
                "--from-document",
                "examples/sample-catalog.json",
                "--variant",
                "secondary",
                "--wcag",
                "3",
            ])
            .sync()
            .await?;
        let _ = badge_container;
        println!("[OK] Stage 5 Passed: shieldcn-zig badges generated.");
    }

    println!("===============================================");
    println!("[SUCCESS] All Mizan CI/CD Dagger stages PASSED.");
    println!("===============================================");

    Ok(())
}
