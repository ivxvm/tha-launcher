#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_derive::{Deserialize, Serialize};
use slint::{ComponentHandle, Timer, TimerMode};
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

slint::include_modules!();

#[derive(Default, Debug, Serialize, Deserialize)]
struct AppConfig {
    is_fullscreen: bool,
    should_show_fps: bool,
}

const APP_NAME: &str = "Templar's Hollow Alpha";

lazy_static::lazy_static! {
    static ref BLENDERPLAYER_PATH: PathBuf = std::env::current_dir().unwrap().join("blenderplayer").join("blenderplayer.exe");
    static ref GAME_STATS_PATH: PathBuf = std::env::current_dir().unwrap().join("gamestats.txt");
}

fn to_model_rc<T: 'static + Clone>(vec: Vec<T>) -> slint::ModelRc<T> {
    slint::ModelRc::new(slint::VecModel::from(vec))
}

fn parse_game_stats() -> Result<Vec<GameStats>, Box<dyn Error>> {
    let stats_file_path = GAME_STATS_PATH.as_path();

    // Create an empty game stats file if it doesn't exist
    if !stats_file_path.exists() {
        std::fs::File::create(stats_file_path)?;
    }

    let mut game_stats = Vec::new();

    for (index, level_stats_str) in std::fs::read_to_string(stats_file_path)?
        .lines()
        .enumerate()
    {
        let mut time = 0;
        let mut gems = 0;
        let mut total_gems = 0;
        let mut unlocked_drawings = vec![false; 3];

        if let [general_stats_str, unlocked_drawings_str, ..] =
            level_stats_str.split(";").collect::<Vec<&str>>()[..]
        {
            // Parse general stats values
            if let [time_str, gems_str, total_gems_str, ..] =
                general_stats_str.split(",").collect::<Vec<&str>>()[..]
            {
                time = time_str.parse().unwrap_or(0);
                gems = gems_str.parse().unwrap_or(0);
                total_gems = total_gems_str.parse().unwrap_or(0);
            }

            // Parse unlocked drawing IDs into a boolean vector
            for drawing_id in unlocked_drawings_str
                .split(",")
                .filter_map(|id| id.parse::<usize>().ok())
            {
                if drawing_id < 3 {
                    unlocked_drawings[drawing_id] = true;
                }
            }
        }

        // Compute drawings based on unlocked_drawings
        let drawings = unlocked_drawings
            .iter()
            .filter(|&&unlocked| unlocked)
            .count() as i32;

        let total_drawings = match index {
            0 => 3,
            1 => 2,
            2 => 2,
            _ => 0,
        };

        // Create a GameStats instance for the level
        game_stats.push(GameStats {
            time,
            gems,
            total_gems,
            drawings,
            total_drawings,
            unlocked_drawings: to_model_rc(unlocked_drawings),
        });
    }

    Ok(game_stats)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut cfg: AppConfig = confy::load(APP_NAME, None)?;
    let game_stats = parse_game_stats()?;

    let app_window = AppWindow::new()?;
    app_window.set_is_fullscreen(cfg.is_fullscreen);
    app_window.set_should_show_fps(cfg.should_show_fps);
    app_window.set_game_stats(to_model_rc(game_stats));

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

            // Launch the game process in a separate thread
            let level_path = app_window.get_level_path().to_string();
            std::thread::spawn({
                let app_window_weak = app_window.as_weak();
                move || {
                    let mut cmd = Command::new(BLENDERPLAYER_PATH.as_path());
                    if let Ok(mut child) = cmd.arg(level_path).args(cmd_args).spawn() {
                        // Wait for the game process to exit
                        if let Err(e) = child.wait() {
                            eprintln!("Failed to wait for game process: {}", e);
                        }

                        // Reload game stats after the game process exits
                        if let Err(e) = slint::invoke_from_event_loop(move || {
                            if let Ok(updated_game_stats) = parse_game_stats() {
                                if let Some(app_window) = app_window_weak.upgrade() {
                                    app_window.set_game_stats(to_model_rc(updated_game_stats));
                                }
                            } else {
                                eprintln!("Failed to parse game stats");
                            }
                        }) {
                            eprintln!("Failed to reload game stats: {}", e);
                        }
                    } else {
                        eprintln!("Failed to launch game");
                    }
                }
            });
        }
    });

    app_window.on_request_open_image({
        move |title, image| {
            let dialog = ImageDialog::new().unwrap();
            dialog.set_label(title);
            dialog.set_image(image);
            dialog.show().unwrap();
        }
    });

    app_window.run()?;
    Ok(())
}
