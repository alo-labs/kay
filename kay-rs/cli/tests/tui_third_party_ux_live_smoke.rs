#![cfg(test)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fmt::Write as _;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;

use code_core::provider_models::{minimax_preset_ids, opencode_go_preset_ids};
use tempfile::TempDir;

mod common;
use common::provider_compat::{
    exact_ok_prompt, shell_tool_prompt, tui_ux_live_smoke_enabled, tui_ux_selected_models,
};
use common::tui_live_harness::{
    assert_session_configured_model, assert_session_start_metadata, code_bin,
    default_live_config_toml, extract_model_header_line, header_shows_exact_model,
    provider_api_key, screen_shows_streaming_content, session_configure_count, turn_timeout,
    wait_for_exact_response, wait_for_exact_response_after_recovery, wait_for_model_header,
    wait_for_screen_contains, DEFAULT_PROMPT_TIMEOUT, TuiLiveHarness, TuiLiveSpawnOptions,
};
use common::SessionPreserver;

#[derive(Clone, Copy)]
struct ThirdPartyUxModelSpec {
    label: &'static str,
    provider_id: &'static str,
    model: &'static str,
    header_label: &'static str,
    bootstrap_model: &'static str,
}

const MINIMAX_IO_M3: ThirdPartyUxModelSpec = ThirdPartyUxModelSpec {
    label: "MiniMax.io MiniMax-M3",
    provider_id: "minimax",
    model: "MiniMax-M3",
    header_label: "MiniMax-M3",
    bootstrap_model: "MiniMax-M2.7",
};

const OPENCODE_GO_MINIMAX_M3: ThirdPartyUxModelSpec = ThirdPartyUxModelSpec {
    label: "OpenCode Go minimax-m3",
    provider_id: "opencode-go",
    model: "opencode-go/minimax-m3",
    header_label: "opencode-go/minimax-m3",
    bootstrap_model: "opencode-go/glm-5.2",
};

fn default_ux_model_specs() -> Vec<ThirdPartyUxModelSpec> {
    let mut specs = Vec::new();
    if minimax_preset_ids()
        .iter()
        .any(|model| model.eq_ignore_ascii_case("MiniMax-M3"))
    {
        specs.push(MINIMAX_IO_M3);
    }
    if opencode_go_preset_ids()
        .iter()
        .any(|model| model.eq_ignore_ascii_case("opencode-go/minimax-m3"))
    {
        specs.push(OPENCODE_GO_MINIMAX_M3);
    }
    specs
}

fn selected_ux_model_specs() -> Vec<ThirdPartyUxModelSpec> {
    let defaults = default_ux_model_specs();
    let default_ids: Vec<String> = defaults
        .iter()
        .map(|spec| spec.model.to_string())
        .collect();
    let selected = tui_ux_selected_models(&default_ids);
    defaults
        .into_iter()
        .filter(|spec| {
            selected.iter().any(|model| {
                model.eq_ignore_ascii_case(spec.model)
                    || model.eq_ignore_ascii_case(spec.header_label)
                    || model.ends_with(spec.model)
            })
        })
        .collect()
}

fn login_provider(kay_home: &TempDir, provider_id: &str, api_key: &str) {
    let mut child = Command::new(code_bin())
        .arg("login")
        .arg("--provider")
        .arg(provider_id)
        .arg("--with-api-key")
        .env("KAY_HOME", kay_home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn kay login");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(api_key.as_bytes())
        .expect("write api key");

    let output = child.wait_with_output().expect("wait for kay login");
    assert!(
        output.status.success(),
        "kay login failed for {provider_id}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn api_key_for_spec(spec: ThirdPartyUxModelSpec) -> Option<String> {
    match spec.provider_id {
        "minimax" => provider_api_key(&["MINIMAX_API_KEY", "MINIMAX_LIVE_API_KEY"], "minimax"),
        "opencode-go" => {
            provider_api_key(&["OPENCODE_GO_API_KEY", "OPENCODE_GO_LIVE_API_KEY"], "opencode-go")
        }
        other => panic!("unexpected provider id for UX smoke: {other}"),
    }
}

#[derive(Default)]
struct JourneyReport {
    launch: bool,
    model_selection: bool,
    basic_chat: bool,
    shell_tool: bool,
    streaming: bool,
    session_metadata: bool,
    error_recovery: bool,
    exit: bool,
}

impl JourneyReport {
    fn all_passed(&self) -> bool {
        self.launch
            && self.model_selection
            && self.basic_chat
            && self.shell_tool
            && self.session_metadata
            && self.error_recovery
            && self.exit
    }

    fn summary(&self, spec: ThirdPartyUxModelSpec) -> String {
        let mut out = format!("\n=== UX journey report: {} ===\n", spec.label);
        for (name, passed) in [
            ("launch", self.launch),
            ("model_selection", self.model_selection),
            ("basic_chat", self.basic_chat),
            ("shell_tool", self.shell_tool),
            ("streaming", self.streaming),
            ("session_metadata", self.session_metadata),
            ("error_recovery", self.error_recovery),
            ("exit", self.exit),
        ] {
            let _ = writeln!(
                &mut out,
                "  {name}: {}",
                if passed { "PASS" } else { "FAIL" }
            );
        }
        out
    }
}

fn start_tui(
    kay_home: &TempDir,
    workspace: &TempDir,
    spec: ThirdPartyUxModelSpec,
) -> TuiLiveHarness {
    TuiLiveHarness::spawn(&TuiLiveSpawnOptions {
        kay_home: kay_home.path().to_path_buf(),
        cwd: workspace.path().to_path_buf(),
        extra_env: Default::default(),
        session_log_path: Some(TuiLiveHarness::session_log_path(kay_home.path())),
        config_toml: Some(default_live_config_toml(
            spec.provider_id,
            spec.bootstrap_model,
        )),
    })
}

fn run_ux_journeys(
    harness: &mut TuiLiveHarness,
    kay_home: &TempDir,
    spec: ThirdPartyUxModelSpec,
    api_key: &str,
) -> JourneyReport {
    let mut report = JourneyReport::default();
    harness.track_secret(api_key.to_string());

    let launch_screen = harness.wait_for(
        &["Model:", "What can I code"],
        DEFAULT_PROMPT_TIMEOUT,
    );
    report.launch = launch_screen.contains("What can I code");
    assert_session_start_metadata(kay_home.path(), spec.provider_id, spec.bootstrap_model);
    println!(
        "UX_LAUNCH[{}] header={}",
        spec.model,
        extract_model_header_line(&launch_screen)
    );

    let configure_count_before = session_configure_count(kay_home.path());
    harness.write_composer_line(&format!("/model {}", spec.model));
    let header_line = wait_for_model_header(harness, spec.header_label, DEFAULT_PROMPT_TIMEOUT);
    report.model_selection = header_shows_exact_model(&header_line, spec.header_label)
        || header_shows_exact_model(&header_line, spec.model);
    println!("UX_MODEL_HEADER[{model}]\n{header_line}\n", model = spec.model);

    let prompt = exact_ok_prompt();
    harness.write_composer_line(prompt);
    let chat_wait = Instant::now();
    let chat_screen = wait_for_exact_response(harness, spec.model, "OK", turn_timeout());
    report.basic_chat = chat_screen.contains("OK");
    println!(
        "UX_BASIC_CHAT[{}] {:.3}s",
        spec.model,
        chat_wait.elapsed().as_secs_f64()
    );

    report.streaming = screen_shows_streaming_content(&chat_screen);
    if !report.streaming {
        println!(
            "UX_STREAMING[{}] soft-check: no explicit reasoning/stream markers detected",
            spec.model
        );
    }

    assert_session_configured_model(
        kay_home.path(),
        configure_count_before,
        spec.provider_id,
        spec.model,
    );
    report.session_metadata = true;

    harness.write_composer_line(shell_tool_prompt());
    let shell_wait = Instant::now();
    let shell_screen = wait_for_screen_contains(harness, "SHELL_OK", turn_timeout());
    let shell_done_screen = wait_for_screen_contains(harness, "DONE", turn_timeout());
    report.shell_tool = shell_screen.contains("SHELL_OK")
        && shell_done_screen.contains("DONE")
        && (shell_screen.contains("echo") || shell_done_screen.contains("echo"));
    println!(
        "UX_SHELL_TOOL[{}] {:.3}s contains_shell_ok={}",
        spec.model,
        shell_wait.elapsed().as_secs_f64(),
        report.shell_tool
    );

    harness.write_composer_line("/model totally-fake-model-xyz");
    let recovery_screen = harness.drain_output_until(DEFAULT_PROMPT_TIMEOUT, |screen| {
        screen.contains("Unknown model preset:")
    });
    harness.write_composer_line(&format!("/model {}", spec.model));
    wait_for_model_header(harness, spec.header_label, DEFAULT_PROMPT_TIMEOUT);
    harness.write_composer_line(exact_ok_prompt());
    let recovered = wait_for_exact_response_after_recovery(harness, spec.model, "OK", turn_timeout());
    report.error_recovery = recovery_screen.contains("Unknown model preset:")
        && recovered.contains("OK")
        && extract_model_header_line(&recovered).contains(spec.header_label);
    println!(
        "UX_ERROR_RECOVERY[{}] recovered={}",
        spec.model, report.error_recovery
    );

    harness.shutdown();
    report.exit = true;
    println!(
        "UX_EXIT[{}] clean shutdown via Ctrl+C",
        spec.model
    );

    report
}

#[test]
fn default_ux_models_include_minimax_io_m3() {
    let specs = default_ux_model_specs();
    assert!(
        specs.iter().any(|spec| spec.model == "MiniMax-M3"),
        "expected MiniMax.io MiniMax-M3 in default UX model set, got {:?}",
        specs.iter().map(|spec| spec.model).collect::<Vec<_>>()
    );
}

#[test]
fn minimax_third_party_tui_ux_live_smoke_matrix() {
    if !tui_ux_live_smoke_enabled() {
        eprintln!("skipping TUI UX live smoke: set KAY_TUI_UX_LIVE_SMOKE=1");
        return;
    }

    let specs = selected_ux_model_specs();
    assert!(
        !specs.is_empty(),
        "no UX model specs selected; defaults cover MiniMax-M3 and opencode-go/minimax-m3"
    );

    let kay_home = TempDir::new().expect("temp KAY_HOME");
    let _sessions = SessionPreserver::new(kay_home.path(), "tui_third_party_ux_live_smoke");
    let workspace = TempDir::new().expect("temp workspace");

    let mut reports = Vec::new();
    for spec in specs {
        let Some(api_key) = api_key_for_spec(spec) else {
            eprintln!(
                "skipping {}: missing API key for provider {}",
                spec.label, spec.provider_id
            );
            continue;
        };

        println!("\n=== TUI UX live smoke: {} ===", spec.label);
        login_provider(&kay_home, spec.provider_id, &api_key);

        let mut harness = start_tui(&kay_home, &workspace, spec);
        let report = run_ux_journeys(&mut harness, &kay_home, spec, &api_key);
        println!("{}", report.summary(spec));
        assert!(
            report.all_passed(),
            "UX journeys failed for {}:\n{}",
            spec.label,
            report.summary(spec)
        );
        reports.push((spec.label, report));
    }

    assert!(
        !reports.is_empty(),
        "no UX live smoke cases ran; configure MINIMAX_API_KEY / MINIMAX_LIVE_API_KEY \
         or provider_credentials.minimax.api_key in ~/.kay/auth.json \
         (and OPENCODE_GO_LIVE_API_KEY for opencode-go/minimax-m3)"
    );
}
