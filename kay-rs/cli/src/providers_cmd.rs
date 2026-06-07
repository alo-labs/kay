use anyhow::Context;
use anyhow::anyhow;
use clap::Parser;
use code_core::config::find_kay_home;
use code_core::model_provider::{
    CompiledHermesProvider, compile_hermes_provider_exports, load_hermes_provider_exports,
};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub(crate) struct ProvidersCommand {
    #[command(subcommand)]
    pub(crate) action: ProvidersAction,
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum ProvidersAction {
    /// Import model-provider profiles from a Hermes Agent checkout.
    #[clap(name = "import-hermes")]
    ImportHermes(ImportHermesCommand),
}

#[derive(Debug, Parser)]
pub(crate) struct ImportHermesCommand {
    /// Path to a Hermes Agent checkout.
    #[arg(long, value_name = "PATH")]
    source: PathBuf,

    /// Write compiled provider profiles to $KAY_HOME/provider_profiles/hermes.
    #[arg(long, default_value_t = false, conflicts_with = "check")]
    install: bool,

    /// Report import status without writing provider profile files.
    #[arg(long, default_value_t = false)]
    check: bool,

    /// Return a non-zero exit code when any provider requires a missing Kay adapter.
    #[arg(long, default_value_t = false)]
    strict: bool,
}

pub(crate) fn run_providers_command(command: ProvidersCommand) -> anyhow::Result<()> {
    match command.action {
        ProvidersAction::ImportHermes(args) => run_import_hermes(args),
    }
}

fn run_import_hermes(args: ImportHermesCommand) -> anyhow::Result<()> {
    if !args.install && !args.check {
        return Err(anyhow!(
            "choose either --install to write profiles or --check to report import status"
        ));
    }

    let source = args
        .source
        .canonicalize()
        .with_context(|| format!("failed to resolve Hermes source {}", args.source.display()))?;
    let exports = load_hermes_provider_exports(&source)
        .with_context(|| format!("failed to export Hermes providers from {}", source.display()))?;
    let total = exports.len();
    let compiled = compile_hermes_provider_exports(exports);

    let mut ready = Vec::new();
    let mut needs_adapter = Vec::new();
    let mut failed = Vec::new();
    for result in compiled {
        match result {
            Ok(provider) if provider.requires_adapter.is_empty() => ready.push(provider),
            Ok(provider) => needs_adapter.push(provider),
            Err(err) => failed.push(err.to_string()),
        }
    }

    if args.install {
        let code_home = find_kay_home().context("failed to locate Kay home")?;
        let install_dir = code_home.join("provider_profiles").join("hermes");
        fs::create_dir_all(&install_dir).with_context(|| {
            format!(
                "failed to create Hermes provider profile directory {}",
                install_dir.display()
            )
        })?;

        for provider in ready.iter().chain(needs_adapter.iter()) {
            write_compiled_profile(&install_dir, provider)?;
        }
    }

    println!(
        "Hermes provider import: {ready_count} ready, {needs_count} require adapters, {failed_count} failed, {total} total",
        ready_count = ready.len(),
        needs_count = needs_adapter.len(),
        failed_count = failed.len(),
    );
    for provider in &needs_adapter {
        println!(
            "requires adapter: {} ({})",
            provider.profile.id,
            provider.requires_adapter.join(", ")
        );
    }
    for error in &failed {
        println!("failed: {error}");
    }

    if args.strict && (!needs_adapter.is_empty() || !failed.is_empty()) {
        return Err(anyhow!(
            "Hermes import requires adapters for {} provider(s) and failed to compile {} provider(s)",
            needs_adapter.len(),
            failed.len()
        ));
    }

    Ok(())
}

fn write_compiled_profile(dir: &std::path::Path, provider: &CompiledHermesProvider) -> anyhow::Result<()> {
    let path = dir.join(format!("{}.json", provider.profile.id));
    let json = serde_json::to_string_pretty(&provider.profile)
        .context("failed to serialize compiled provider profile")?;
    fs::write(&path, format!("{json}\n"))
        .with_context(|| format!("failed to write provider profile {}", path.display()))?;
    Ok(())
}

