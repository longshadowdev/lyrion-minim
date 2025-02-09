// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use tauri::{Manager, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tauri_plugin_positioner::{Position, WindowExt};
use std::process::Command;

#[tauri::command]
fn get_config() -> HashMap<String, String> {
    // User configuration
    let config = HashMap::from([
        ("clientName".to_owned(), "Lyrion Minim".to_owned()),
        ("squeezelitePath".to_owned(), "/Applications/SqueezeLite.app/Contents/MacOS/SqueezeLite".to_owned()),
        ("lyrionBaseUrl".to_owned(), "http://192.168.1.2:9000".to_owned()),
    ]);
    
    return config;
}

fn main() {

    let config = get_config();

    // Start Squeezelite
    let squeezelite_pid = Command::new(config.get("squeezelitePath").unwrap())
        .args(["-n", config.get("clientName").unwrap()])
        .spawn()
        .expect("failed to start Squeezelite")
        .id();

    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .system_tray(
            SystemTray::new().with_menu(
                SystemTrayMenu::new()
                    .add_item(
                        CustomMenuItem::new("quit", "Quit")
                            .accelerator("Cmd+Q")
                    )
            )
        )
        .on_system_tray_event(move |app, event| {
            tauri_plugin_positioner::on_tray_event(app, &event);
            match event {
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
                    "quit" => {
                        // Kill the squeezelite process
                        Command::new("kill").arg(squeezelite_pid.to_string()).spawn().expect("failed to execute process");
                        // Exit Lyrion Minim
                        std::process::exit(0);
                    }
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
        .invoke_handler(tauri::generate_handler![greet, get_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
