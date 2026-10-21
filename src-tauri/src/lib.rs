mod model;
mod adapt;
mod db;
mod export;

use model::{load_task, HintLevel, Task};
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Mutex;
use tauri::Manager;

struct AppState {
    db: Mutex<Connection>,
}

#[derive(Serialize)]
struct LapResult {
    next: HintLevel,
    clean_streak: u32,
    advanced: bool,
    needed: u32,
}

#[tauri::command]
fn get_task() -> Result<Task, String> {
    load_task(include_str!("../../tasks/wash_hands.json")).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_players(state: tauri::State<AppState>) -> Vec<String> {
    let conn = state.db.lock().unwrap();
    db::list_players(&conn).unwrap_or_default()
}

#[tauri::command]
fn add_player(name: String, state: tauri::State<AppState>) -> Vec<String> {
    let name = name.trim().to_string();
    let conn = state.db.lock().unwrap();
    if !name.is_empty() {
        let _ = db::add_player(&conn, &name);
    }
    db::list_players(&conn).unwrap_or_default()
}

#[tauri::command]
fn start_player(name: String, state: tauri::State<AppState>) -> HintLevel {
    let conn = state.db.lock().unwrap();
    db::player_state(&conn, &name)
        .ok()
        .flatten()
        .map(|(level, _)| level)
        .unwrap_or(HintLevel::Video)
}

#[tauri::command]
fn finish_lap(
    player: String,
    level: HintLevel,
    errors: u32,
    seconds: u32,
    state: tauri::State<AppState>,
) -> LapResult {
    let conn = state.db.lock().unwrap();

    // серия считается только если играли на том же уровне, что сохранён
    let streak_before = match db::player_state(&conn, &player) {
        Ok(Some((saved_level, streak))) if saved_level == level => streak,
        _ => 0,
    };

    let out = adapt::after_lap(level, streak_before, errors);
    let _ = db::save_session(&conn, &player, level, errors, seconds);
    let _ = db::set_player_state(&conn, &player, out.level, out.clean_streak);

    LapResult {
        next: out.level,
        clean_streak: out.clean_streak,
        advanced: out.advanced,
        needed: adapt::CLEAN_LAPS_TO_ADVANCE,
    }
}

#[tauri::command]
fn delete_player(name: String, state: tauri::State<AppState>) -> Vec<String> {
    let conn = state.db.lock().unwrap();
    let _ = db::delete_player(&conn, &name);
    db::list_players(&conn).unwrap_or_default()
}


#[tauri::command]
fn export_excel(app: tauri::AppHandle, state: tauri::State<AppState>) -> Result<String, String> {
    let (sessions, summary, stamp) = {
        let conn = state.db.lock().unwrap();
        (
            db::all_sessions(&conn).map_err(|e| e.to_string())?,
            db::summary(&conn).map_err(|e| e.to_string())?,
            db::file_stamp(&conn).map_err(|e| e.to_string())?,
        )
    };
    if sessions.is_empty() {
        return Err("Nessun dato da esportare".into());
    }

    let dir = app
        .path()
        .document_dir()
        .or_else(|_| app.path().home_dir())
        .map_err(|e| e.to_string())?;
    let path = dir.join(format!("hands_{stamp}.xlsx"));

    export::write_xlsx(&path, &sessions, &summary).map_err(|e| e.to_string())?;
    Ok(path.display().to_string())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = db::open(&dir.join("hands.db"))?;
            app.manage(AppState { db: Mutex::new(conn) });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_task, list_players, add_player, delete_player, start_player, finish_lap, export_excel
        ])
        .run(tauri::generate_context!())
        .expect("error");
}