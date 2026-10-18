use super::*;

#[tokio::test]
async fn direct_and_packed_previews_and_writes_share_revision_and_publication_contracts() {
    for route in [Route::Direct, Route::Packed] {
        for name in WRITE_TOOLS {
            let fixture = Fixture::new(AutonomyLevel::Full);
            let mut args = fixture.mutation_args(name, true).await;
            let preview = fixture.json(route, name, args.clone()).await;
            assert_eq!(preview["written"], false);
            assert_eq!(preview["dry_run"], true);
            assert_eq!(preview["changed"], true);
            fixture.assert_original_files();
            args["dry_run"] = json!(false);
            let written = fixture.json(route, name, args.clone()).await;
            assert_eq!(written["written"], true);
            assert_eq!(written["dry_run"], false);
            assert_eq!(written["revision"], preview["revision"]);
            let path = fixture.project.join(if name == "ast_patch" {
                "source.rs"
            } else {
                "new.rs"
            });
            let published = std::fs::read_to_string(&path).unwrap();
            assert_eq!(written["source"]["text"], published);
            assert!(published.contains(if name == "ast_patch" {
                "2"
            } else {
                "fn created"
            }));
            let retry = fixture.call(route, name, args).await;
            assert!(
                retry.is_error,
                "stale replacement or create-new retry must refuse"
            );
            assert_eq!(std::fs::read_to_string(path).unwrap(), published);
            assert_eq!(
                std::fs::read_dir(&fixture.project).unwrap().count(),
                if name == "ast_patch" { 1 } else { 2 }
            );
        }
    }
}

#[tokio::test]
async fn read_only_security_accepts_literal_previews_and_refuses_writes_without_artifacts() {
    for route in [Route::Direct, Route::Packed] {
        let fixture = Fixture::new(AutonomyLevel::ReadOnly);
        for name in WRITE_TOOLS {
            let mut args = fixture.mutation_args(name, true).await;
            assert_eq!(
                fixture.json(route, name, args.clone()).await["written"],
                false
            );
            for dry_run in [json!(false), json!("true"), Value::Null] {
                args["dry_run"] = dry_run;
                let denied = fixture.call(route, name, args.clone()).await;
                assert!(
                    denied.is_error,
                    "{route:?} {name} must refuse a non-preview"
                );
                fixture.assert_original_files();
            }
            args.as_object_mut().unwrap().remove("dry_run");
            assert!(fixture.call(route, name, args).await.is_error);
            fixture.assert_original_files();
        }
    }
}

#[tokio::test]
async fn stale_content_and_uninspected_chunks_never_modify_the_destination() {
    for route in [Route::Direct, Route::Packed] {
        let fixture = Fixture::new(AutonomyLevel::Full);
        let args = fixture.mutation_args("ast_patch", false).await;
        let edited = "fn value() -> i32 { 99 }\n";
        std::fs::write(fixture.project.join("source.rs"), edited).unwrap();
        let stale = fixture.call(route, "ast_patch", args).await;
        assert!(stale.is_error && stale.output().contains("revision"));
        assert_eq!(
            std::fs::read_to_string(fixture.project.join("source.rs")).unwrap(),
            edited
        );
        let mut args = fixture.mutation_args("chunk_apply", false).await;
        args["chunk_revision"] = json!("a".repeat(64));
        let stale = fixture.call(route, "chunk_apply", args).await;
        assert!(stale.is_error && stale.output().contains("chunk revision"));
        assert!(!fixture.project.join("new.rs").exists());
        assert_eq!(std::fs::read_dir(&fixture.project).unwrap().count(), 1);
    }
}

#[tokio::test]
async fn only_the_trusted_run_context_grants_the_selected_project_root() {
    let fixture = Fixture::new(AutonomyLevel::Full);
    let absolute = fixture.project.join("source.rs");
    let denied = fixture
        .tool("ast_outline")
        .execute(json!({"path": absolute}))
        .await
        .unwrap();
    assert!(
        denied.is_error,
        "the host action directory is separate from the selected project"
    );
    let spoofed = fixture
        .tool("ast_outline")
        .execute(json!({
            "path": absolute, "workspace_root": fixture.project
        }))
        .await
        .unwrap();
    assert!(
        spoofed.is_error,
        "model arguments cannot supply a trusted workspace"
    );
    let allowed = fixture
        .json(Route::Packed, "ast_outline", json!({"path": "source.rs"}))
        .await;
    assert_eq!(allowed["symbols"]["items"][0]["name"], "value");
    let sibling = fixture._root.path().join("sibling.rs");
    std::fs::write(&sibling, "fn sibling() {}\n").unwrap();
    assert!(
        fixture
            .call(Route::Packed, "ast_outline", json!({"path": sibling}))
            .await
            .is_error
    );
    fixture.assert_original_files();
}

#[tokio::test]
async fn trusted_roots_keep_credentials_and_host_state_inaccessible() {
    let fixture = Fixture::new(AutonomyLevel::Full);
    let credentials = fixture.project.join(".ssh");
    std::fs::create_dir(&credentials).unwrap();
    let private = credentials.join("secret.rs");
    std::fs::write(&private, "fn secret() {}\n").unwrap();
    for route in [Route::Direct, Route::Packed] {
        assert!(
            fixture
                .call(route, "ast_outline", json!({"path": private}))
                .await
                .is_error
        );
    }
    let sessions = fixture.home.join("sessions");
    std::fs::create_dir(&sessions).unwrap();
    let state = sessions.join("state.rs");
    std::fs::write(&state, "fn internal() {}\n").unwrap();
    let context = ToolExecutionContext::from_run_context(
        &Fixture::run_context_at(&fixture.home),
        CallId::new("internal"),
    );
    let denied = fixture
        .tool("ast_outline")
        .execute_with_context(
            json!({"path": state}),
            ToolCallOptions::default(),
            Some(&context),
        )
        .await
        .unwrap();
    assert!(
        denied.is_error,
        "a run workspace cannot grant access to host session state"
    );
    assert_eq!(
        std::fs::read_to_string(private).unwrap(),
        "fn secret() {}\n"
    );
    assert_eq!(
        std::fs::read_to_string(state).unwrap(),
        "fn internal() {}\n"
    );
}
