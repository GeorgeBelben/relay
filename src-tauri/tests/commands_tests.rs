//! Exercises Tauri commands through the real IPC boundary (`tauri::test`), not just as plain
//! Rust functions. This is what actually proves the frontend's camelCase JS argument names
//! (e.g. `retroarchCore`) deserialize into the commands' snake_case Rust parameters --
//! calling the functions directly, as the repository tests do, can't catch a naming mismatch.

use serde_json::{json, Value};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::{Manager, WebviewWindowBuilder};

use relay_lib::commands;

mod common;
use common::throwaway_pool;

fn invoke_request(cmd: &str, args: Value) -> InvokeRequest {
    InvokeRequest {
        cmd: cmd.into(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: if cfg!(any(windows, target_os = "android")) {
            "http://tauri.localhost"
        } else {
            "tauri://localhost"
        }
        .parse()
        .unwrap(),
        body: InvokeBody::Json(args),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    }
}

#[tokio::test]
async fn create_system_command_accepts_camel_case_args() {
    let (pool, _dir) = throwaway_pool().await;

    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            commands::systems::create_system,
            commands::systems::get_system,
        ])
        .build(mock_context(noop_assets()))
        .expect("failed to build mock app");
    app.manage(pool);

    let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let create_res = get_ipc_response(
        &webview,
        invoke_request(
            "create_system",
            json!({
                "id": "nes",
                "name": "NES",
                "extensions": "[\"nes\"]",
                "retroarchCore": "mesen",
                "standaloneBinary": null,
            }),
        ),
    );
    let created: Value = create_res.expect("create_system should succeed").deserialize().unwrap();
    assert_eq!(created["id"], "nes");
    assert_eq!(created["retroarch_core"], "mesen");

    let get_res = get_ipc_response(&webview, invoke_request("get_system", json!({ "id": "nes" })));
    let fetched: Value = get_res.expect("get_system should succeed").deserialize().unwrap();
    assert_eq!(fetched["name"], "NES");
}

#[tokio::test]
async fn create_rom_and_game_commands_accept_camel_case_args() {
    let (pool, _dir) = throwaway_pool().await;

    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            commands::systems::create_system,
            commands::roms::create_rom,
            commands::games::create_game,
            commands::games::list_games,
        ])
        .build(mock_context(noop_assets()))
        .expect("failed to build mock app");
    app.manage(pool);

    let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    get_ipc_response(
        &webview,
        invoke_request(
            "create_system",
            json!({ "id": "snes", "name": "SNES", "extensions": "[\"sfc\"]", "retroarchCore": null, "standaloneBinary": null }),
        ),
    )
    .expect("create_system should succeed");

    let rom_res = get_ipc_response(
        &webview,
        invoke_request(
            "create_rom",
            json!({
                "systemId": "snes",
                "path": "snes/game.sfc",
                "crc32": null,
                "sizeBytes": null,
                "discs": null,
            }),
        ),
    );
    let rom: Value = rom_res.expect("create_rom should succeed").deserialize().unwrap();
    let rom_id = rom["id"].as_str().unwrap().to_string();

    let game_res = get_ipc_response(
        &webview,
        invoke_request("create_game", json!({ "romId": rom_id, "title": "A Game" })),
    );
    let game: Value = game_res.expect("create_game should succeed").deserialize().unwrap();
    assert_eq!(game["title"], "A Game");

    let list_res = get_ipc_response(&webview, invoke_request("list_games", json!({})));
    let list: Value = list_res.expect("list_games should succeed").deserialize().unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn settings_commands_round_trip_through_ipc() {
    let (pool, _dir) = throwaway_pool().await;

    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_setting,
            commands::settings::set_setting,
        ])
        .build(mock_context(noop_assets()))
        .expect("failed to build mock app");
    app.manage(pool);

    let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let missing_res = get_ipc_response(
        &webview,
        invoke_request("get_setting", json!({ "key": "steamgriddbApiKey" })),
    );
    let missing: Value = missing_res.expect("get_setting should succeed").deserialize().unwrap();
    assert!(missing.is_null());

    let set_res = get_ipc_response(
        &webview,
        invoke_request("set_setting", json!({ "key": "steamgriddbApiKey", "value": "abc123" })),
    );
    assert!(set_res.is_ok(), "set_setting failed: {:?}", set_res);

    let get_res = get_ipc_response(
        &webview,
        invoke_request("get_setting", json!({ "key": "steamgriddbApiKey" })),
    );
    let value: Value = get_res.expect("get_setting should succeed").deserialize().unwrap();
    assert_eq!(value, "abc123");
}

#[tokio::test]
async fn launch_game_command_spawns_via_the_real_ipc_boundary_and_reports_status() {
    let (pool, _dir) = throwaway_pool().await;

    // "true" stands in for a real emulator binary -- exists on every Unix dev/CI machine, exits
    // 0 immediately regardless of arguments, so this proves the full DB-lookup -> build-command ->
    // spawn -> status pipeline without needing a real RetroArch/PCSX2/Dolphin install.
    relay_lib::db::systems::create(
        &pool,
        relay_lib::db::systems::NewSystem {
            id: "snes".into(),
            name: "SNES".into(),
            extensions: r#"["sfc"]"#.into(),
            retroarch_core: None,
            standalone_binary: Some("true".into()),
        },
    )
    .await
    .unwrap();
    let rom = relay_lib::db::roms::create(
        &pool,
        relay_lib::db::roms::NewRom { system_id: "snes".into(), path: "snes/game.sfc".into(), crc32: None, size_bytes: None, discs: None },
    )
    .await
    .unwrap();
    let game =
        relay_lib::db::games::create(&pool, relay_lib::db::games::NewGame { rom_id: rom.id, title: "A Game".into() }).await.unwrap();

    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            commands::emulator::launch_game,
            commands::emulator::get_launcher_status,
            commands::emulator::kill_game,
        ])
        .build(mock_context(noop_assets()))
        .expect("failed to build mock app");
    app.manage(pool);
    app.manage(commands::emulator::LauncherState::default());

    let webview = WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();

    let launch_res = get_ipc_response(&webview, invoke_request("launch_game", json!({ "gameId": game.id })));
    assert!(launch_res.is_ok(), "launch_game failed: {:?}", launch_res);

    // launch_game awaits the whole process lifecycle before resolving (matching
    // ingestion::pipeline::rescan's precedent), so by the time it returns, "true" has already
    // exited and get_launcher_status should reflect the final state.
    let status_res = get_ipc_response(&webview, invoke_request("get_launcher_status", json!({})));
    let status: Value = status_res.expect("get_launcher_status should succeed").deserialize().unwrap();
    assert_eq!(status["state"], "exited");

    // Nothing is running any more, so kill_game should report that rather than silently no-op.
    let kill_res = get_ipc_response(&webview, invoke_request("kill_game", json!({})));
    assert!(kill_res.is_err(), "expected kill_game to fail when nothing is running");
}

#[tokio::test]
async fn general_settings_commands_round_trip_through_ipc_with_camel_case_args() {
    let (pool, _dir) = throwaway_pool().await;

    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_general_settings,
            commands::settings::set_controller_type,
            commands::settings::set_active_profile_id,
            commands::settings::set_sound_volume,
        ])
        .build(mock_context(noop_assets()))
        .expect("failed to build mock app");
    app.manage(pool);

    let webview = WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();

    let defaults_res = get_ipc_response(&webview, invoke_request("get_general_settings", json!({})));
    let defaults: Value = defaults_res.expect("get_general_settings should succeed").deserialize().unwrap();
    assert_eq!(defaults["controller_type"], "xbox");
    assert_eq!(defaults["sound_volume"], 70);
    assert!(defaults["active_profile_id"].is_null());

    let set_controller_res = get_ipc_response(
        &webview,
        invoke_request("set_controller_type", json!({ "controllerType": "playstation" })),
    );
    assert!(set_controller_res.is_ok(), "set_controller_type failed: {:?}", set_controller_res);

    let set_profile_res =
        get_ipc_response(&webview, invoke_request("set_active_profile_id", json!({ "profileId": "profile-1" })));
    assert!(set_profile_res.is_ok(), "set_active_profile_id failed: {:?}", set_profile_res);

    let set_volume_res = get_ipc_response(&webview, invoke_request("set_sound_volume", json!({ "volume": 42 })));
    assert!(set_volume_res.is_ok(), "set_sound_volume failed: {:?}", set_volume_res);

    let updated_res = get_ipc_response(&webview, invoke_request("get_general_settings", json!({})));
    let updated: Value = updated_res.expect("get_general_settings should succeed").deserialize().unwrap();
    assert_eq!(updated["controller_type"], "playstation");
    assert_eq!(updated["active_profile_id"], "profile-1");
    assert_eq!(updated["sound_volume"], 42);

    // null clears it back to "no active profile" -- Option<String> deserializing from a JS null.
    let clear_res = get_ipc_response(&webview, invoke_request("set_active_profile_id", json!({ "profileId": null })));
    assert!(clear_res.is_ok(), "clearing active profile failed: {:?}", clear_res);
    let cleared: Value = get_ipc_response(&webview, invoke_request("get_general_settings", json!({})))
        .unwrap()
        .deserialize()
        .unwrap();
    assert!(cleared["active_profile_id"].is_null());
}
