use agileplus_cli::commands::triage::{TriageArgs, run_triage};

#[tokio::test]
async fn triage_command_runs_without_error() {
    let args = TriageArgs {
        input: vec!["Crash on login".to_string(), "OAuth callback fails".to_string()],
        r#type: Some("bug".to_string()),
        dry_run: true,
        output: "json".to_string(),
    };
    let result = run_triage(args).await;
    assert!(result.is_ok(), "triage command should succeed: {:?}", result.err());
}
