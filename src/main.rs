#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::process::Command;

slint::include_modules!();

const BLENDERPLAYER_PATH: &str = "C:\\Programs\\upbge-0.36.1-windows-x86_64\\blenderplayer.exe";

fn main() -> Result<(), Box<dyn Error>> {
    let app_window = AppWindow::new()?;
    let app_window_weak = app_window.as_weak();

    app_window.on_request_exit(|| {
        std::process::exit(0);
    });

    app_window.on_request_play_level(move || {
        let app_window = app_window_weak.unwrap();
        let mut cmd_args = vec!["-", "-launcher"];
        if app_window.get_is_fullscreen() {
            cmd_args.push("-fullscreen");
        }
        if app_window.get_should_show_fps() {
            cmd_args.push("-fps");
        }
        println!("args: {:?}", cmd_args);
        let mut cmd = Command::new(BLENDERPLAYER_PATH);
        cmd.arg(app_window.get_level_path())
            .args(cmd_args)
            .spawn()
            .unwrap();
    });

    app_window.run()?;
    Ok(())
}
