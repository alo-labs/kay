#![cfg(test)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;

use code_core::model_family::wire_model_slug;
use code_core::provider_models::opencode_go_preset_ids;
use tempfile::TempDir;

mod common;
use common::provider_compat::{
    assert_wire_profile, compatibility_profile, exact_ok_prompt, tui_live_smoke_enabled,
    tui_selected_models,
};
use common::tui_live_harness::{
    assert_session_configured_model, code_bin, default_live_config_toml, env_key,
    session_configure_count, turn_timeout, wait_for_exact_response, wait_for_model_header,
    DEFAULT_PROMPT_TIMEOUT, TuiLiveHarness, TuiLiveSpawnOptions,
};
use common::SessionPreserver;

const PROVIDER_ID: &str = "opencode-go";

const DEFAULT_TUI_LIVE_MODELS: &[&str] = &[
    "opencode-go/glm-5.2",
    "opencode-go/deepseek-v4-pro",
    "opencode-go/deepseek-v4-flash",
];

fn default_tui_live_models() -> Vec<String> {
    let all = opencode_go_preset_ids();
    let defaults: Vec<String> = DEFAULT_TUI_LIVE_MODELS
        .iter()
        .filter_map(|wanted| {
            all.iter()
                .find(|model| model.eq_ignore_ascii_case(wanted))
                .cloned()
        })
        .collect();
    if defaults.is_empty() {
        all.into_iter().take(3).collect()
    } else {
        defaults
    }
}

fn login_provider(kay_home: &TempDir, api_key: &str) {
    let mut child = Command::new(code_bin())
        .arg("login")
        .arg("--provider")
        .arg(PROVIDER_ID)
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
        "kay login failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn bootstrap_model_for(target_model: &str) -> &'static str {
    if target_model.eq_ignore_ascii_case("opencode-go/kimi-k2.6") {
        "opencode-go/glm-5.2"
    } else {
        "opencode-go/kimi-k2.6"
    }
}

fn start_configured_tui(kay_home: &TempDir, workspace: &TempDir, model: &str) -> TuiLiveHarness {
    let bootstrap_model = bootstrap_model_for(model);
    let mut harness = TuiLiveHarness::spawn(&TuiLiveSpawnOptions {
        kay_home: kay_home.path().to_path_buf(),
        cwd: workspace.path().to_path_buf(),
        extra_env: Default::default(),
        session_log_path: Some(TuiLiveHarness::session_log_path(kay_home.path())),
        config_toml: Some(default_live_config_toml(PROVIDER_ID, bootstrap_model)),
    });
    harness.wait_for(
        &["Model:", "What can I code"],
        DEFAULT_PROMPT_TIMEOUT,
    );
    harness
}

fn run_basic_tui_turn(
    harness: &mut TuiLiveHarness,
    kay_home: &TempDir,
    model: &str,
) {
    let configure_count_before = session_configure_count(kay_home.path());
    harness.write_composer_line(&format!("/model {model}"));
    let header_line = wait_for_model_header(harness, model, DEFAULT_PROMPT_TIMEOUT);
    println!("\nTUI_LIVE_HEADER[{model}]\n{header_line}\n");

    let prompt = exact_ok_prompt();
    println!("TUI_LIVE_PROMPT[{model}]\n{prompt}\n");
    harness.write_composer_line(prompt);

    let response_wait = Instant::now();
    let screen = wait_for_exact_response(harness, model, "OK", turn_timeout());
    let elapsed = response_wait.elapsed();
    println!(
        "TUI_LIVE_RESPONSE_WAIT[{model}]\n{:.3}s\n",
        elapsed.as_secs_f64()
    );
    println!(
        "TUI_LIVE_SCREEN_TAIL[{model}]\n{}\n",
        screen.lines().rev().take(8).collect::<Vec<_>>().join("\n")
    );

    assert_session_configured_model(
        kay_home.path(),
        configure_count_before,
        PROVIDER_ID,
        model,
    );

    let profile = compatibility_profile(PROVIDER_ID, model);
    assert_wire_profile(PROVIDER_ID, model, &profile);
    assert_eq!(
        wire_model_slug(PROVIDER_ID, model),
        profile.expected_wire_slug,
        "wire slug mismatch for {model}"
    );
}

#[test]
fn default_tui_live_models_are_curated_opencode_go_proof_set() {
    let models = default_tui_live_models();
    assert!(
        models
            .iter()
            .any(|model| model.eq_ignore_ascii_case("opencode-go/glm-5.2")),
        "expected glm-5.2 in default TUI live set, got {models:?}"
    );
    assert!(models.iter().all(|model| model.starts_with("opencode-go/")));
}

#[test]
fn opencode_go_tui_provider_live_smoke_matrix() {
    if !tui_live_smoke_enabled() {
        eprintln!("skipping TUI provider live smoke: set KAY_TUI_PROVIDER_LIVE_SMOKE=1");
        return;
    }

    let Some(api_key) = env_key("OPENCODE_GO_LIVE_API_KEY") else {
        eprintln!(
            "skipping TUI provider live smoke: set OPENCODE_GO_LIVE_API_KEY (or configure provider_credentials.opencode-go.api_key in KAY_HOME)"
        );
        return;
    };

    let models = tui_selected_models(&default_tui_live_models());
    assert!(
        !models.is_empty(),
        "no models selected for TUI provider live smoke"
    );

    let kay_home = TempDir::new().expect("temp KAY_HOME");
    let _sessions = SessionPreserver::new(kay_home.path(), "tui_provider_live_smoke");
    let workspace = TempDir::new().expect("temp workspace");
    login_provider(&kay_home, &api_key);

    for model in models {
        println!("\n=== TUI live smoke: {model} ===");
        let mut harness = start_configured_tui(&kay_home, &workspace, &model);
        harness.track_secret(api_key.clone());
        run_basic_tui_turn(&mut harness, &kay_home, &model);
    }
}
