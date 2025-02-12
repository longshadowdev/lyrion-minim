// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tauri_plugin_positioner::{Position, WindowExt};
use std::time::Duration;
use std::format;
use discover::discover;
mod discover;

#[tauri::command]
async fn detect_lms_server() -> String {
    let reply = discover(Duration::from_secs(5)).await.unwrap();
    return format!("{}:{}", reply.hostname, reply.port);
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .system_tray(
            SystemTray::new().with_menu(
                SystemTrayMenu::new()
                    .add_item(
                        CustomMenuItem::new("quit", "Quit")
                            .accelerator("Cmd+Q")
                    ).add_item(
                        CustomMenuItem::new("settings", "Settings")
                    )
            )
        )
        .on_system_tray_event(move |app, event| {
            tauri_plugin_positioner::on_tray_event(app, &event);
            match event {
                // User has clicked the tray icon
                SystemTrayEvent::LeftClick {
                    position: _,
                    size: _,
                    ..
                } => {
                    let window = app.get_window("main").unwrap();
                    // use TrayCenter as initial window position
                    let _ = window.move_window(Position::TrayCenter);
                    if window.is_visible().unwrap() {
                        window.hide().unwrap();
                    } else {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                    // User has clicked the quit menu item
                    "quit" => {
                        app.get_window("main")
                            .unwrap()
                            .emit("quit", ())
                            .unwrap();
                    },
                    // User has clicked the settings menu item
                    "settings" => {
                        app.get_window("main")
                            .unwrap()
                            .emit("settings", ())
                            .unwrap();
                    },
                    _ => {}
                }
                _ => {}
            }
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::Focused(is_focused) => {
                // detect click outside of the focused window and hide the app
                if !is_focused {
                    event.window().hide().unwrap();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![detect_lms_server])
        .run(tauri::generate_context!())
        .expect("Error while running Lyrion Minim");
}
