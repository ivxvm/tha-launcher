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
const GAME_STATS_PATH: &str = "E:\\Blender\\Nodot\\gamestats.txt";

fn parse_game_stats() -> Result<Vec<slint_generatedAppWindow::GameStats>, Box<dyn Error>> {
    let mut game_stats = Vec::new();

    for level_stats_str in std::fs::read_to_string(GAME_STATS_PATH)?.lines() {
        if let [general_stats_str, unlocked_drawings_str, ..] =
            level_stats_str.split(";").collect::<Vec<&str>>()[..]
        {
            if let [time_str, gems_str, total_gems_str, drawings_str, total_drawings_str, ..] =
                general_stats_str.split(",").collect::<Vec<&str>>()[..]
            {
                // Parse general stats values
                let time: i32 = time_str.parse()?;
                let gems: i32 = gems_str.parse()?;
                let total_gems: i32 = total_gems_str.parse()?;
                let drawings: i32 = drawings_str.parse()?;
                let total_drawings: i32 = total_drawings_str.parse()?;

                // Parse unlocked drawing IDs into a boolean vector
                let mut unlocked_drawings = vec![false; 3];
                for drawing_id in unlocked_drawings_str
                    .split(",")
                    .filter_map(|id| id.parse::<usize>().ok())
                {
                    if drawing_id < 3 {
                        unlocked_drawings[drawing_id] = true;
                    }
                }

                // Create a GameStats instance for the level
                game_stats.push(GameStats {
                    time,
                    gems,
                    total_gems,
                    drawings,
                    total_drawings,
                    unlocked_drawings: slint::ModelRc::new(slint::VecModel::from(
                        unlocked_drawings,
                    )),
                });
            }
        }
    }

    Ok(game_stats)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut cfg: AppConfig = confy::load(APP_NAME, None)?;
    let game_stats = parse_game_stats()?;

    let app_window = AppWindow::new()?;
    app_window.set_is_fullscreen(cfg.is_fullscreen);
    app_window.set_should_show_fps(cfg.should_show_fps);
    app_window.set_game_stats(slint::ModelRc::new(slint::VecModel::from(game_stats)));

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

    app_window.on_request_open_url({
        let app_window_weak = app_window.as_weak();
        move || {
            let app_window = app_window_weak.unwrap();
            open::that(app_window.get_url_to_open()).unwrap();
        }
    });

    app_window.run()?;
    Ok(())
}
