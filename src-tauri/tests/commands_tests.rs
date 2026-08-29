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
