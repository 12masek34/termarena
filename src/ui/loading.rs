use std::sync::Mutex;

use macroquad::prelude::*;

use crate::network::state::MapDownloader;

pub fn draw_loading_screen(loading_frame: u32, map_downloader: &Mutex<MapDownloader>) {
    clear_background(BLACK);

    let loading_text = format!("Loading{}", ".".repeat((loading_frame % 4) as usize));
    draw_text(&loading_text, 20.0, 50.0, 30.0, WHITE);

    let (received, total) = {
        let dl = map_downloader.lock().unwrap();
        dl.progress()
    };

    if total > 0 {
        let percent = received as f32 / total as f32 * 100.0;
        let prog_text = format!("Downloading map: {}/{} ({:.1}%)", received, total, percent);
        draw_text(&prog_text, 20.0, 90.0, 25.0, WHITE);

        let bar_width = 300.0;
        let filled = bar_width * (received as f32 / total as f32);

        draw_rectangle(20.0, 110.0, bar_width, 20.0, GRAY);
        draw_rectangle(20.0, 110.0, filled, 20.0, GREEN);
    }
}
