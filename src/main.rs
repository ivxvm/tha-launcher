#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_derive::{Deserialize, Serialize};
use slint::{Timer, TimerMode};
use std::error::Error;
use std::process::Command;

slint::include_modules!();

#[derive(Default, Debug, Serialize, Deserialize)]
struct AppConfig {
    is_fullscreen: bool,
    should_show_fps: bool,
}

const APP_NAME: &str = "xawn";
const BLENDERPLAYER_PATH: &str = "C:\\Programs\\upbge-0.36.1-windows-x86_64\\blenderplayer.exe";

fn main() -> Result<(), Box<dyn Error>> {
    let mut cfg: AppConfig = confy::load(APP_NAME, None)?;

    let app_window = AppWindow::new()?;
    app_window.set_is_fullscreen(cfg.is_fullscreen);
    app_window.set_should_show_fps(cfg.should_show_fps);

    let cfg_update_timer = Timer::default();
    cfg_update_timer.start(
        TimerMode::Repeated,
        std::time::Duration::from_millis(250),
        {
            let app_window_weak = app_window.as_weak();
            move || {
                let app_window = app_window_weak.unwrap();
                let is_fullscreen = app_window.get_is_fullscreen();
                let should_show_fps = app_window.get_should_show_fps();
                if cfg.is_fullscreen != is_fullscreen || cfg.should_show_fps != should_show_fps {
                    cfg.is_fullscreen = is_fullscreen;
                    cfg.should_show_fps = should_show_fps;
                    confy::store(APP_NAME, None, &cfg).unwrap();
                }
            }
        },
    );

    app_window.on_request_exit(|| {
        std::process::exit(0);
    });

    app_window.on_request_play_level({
        let app_window_weak = app_window.as_weak();
        move || {
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
        }
    });

    app_window.run()?;
    Ok(())
}
