use badapple_player::decode;
use badapple_player::convert;
use badapple_player::render;
use badapple_player::audio;
use badapple_player::terminal_guard::TerminalGuard;

use std::env;
use std::time::{Duration, Instant};
use std::thread;
use crossterm::terminal;
use crossterm::event::{poll, read, Event, KeyCode};

#[derive(Clone, Copy)]
enum RenderMode {
    Otsu,
    Dither,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <path-to-video> <path-to-audio> [otsu|dither]", args[0]);
        return;
    }
    let video_path = args[1].clone();
    let audio_path = args[2].clone();

    let mode = match args.get(3).map(|s| s.as_str()) {
        Some("otsu") => RenderMode::Otsu,
        Some("dither") => RenderMode::Dither,
        Some(other) => {
            eprintln!("Unknown mode '{}', defaulting to dither", other);
            RenderMode::Dither
        }
        None => RenderMode::Dither,
    };

    let (term_width, term_height) = terminal::size().expect("failed to read terminal size");
    let target_width = term_width as u32;
    let target_height = (term_height as u32) * 2;

    thread::spawn(move || {
        audio::play_audio_blocking(&audio_path);
    });

    let _guard = TerminalGuard::new();

    let start_time = Instant::now();
    let mut frame_count: u32 = 0;
    let mut current_threshold: u8 = 128;
    let mut dropped_frames: u32 = 0;
    let mut total_frames: u32 = 0;

    
    decode::decode_all_frames(&video_path, |frame| {
        total_frames += 1;

        if poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(key_event)) = read() {
                if key_event.code == KeyCode::Char('q') || key_event.code == KeyCode::Char('Q') {
                    return false;
                }
            }
        }

        let expected_time = Duration::from_secs_f32(frame.timestamp);
        let elapsed = start_time.elapsed();

        if elapsed < expected_time {
            thread::sleep(expected_time - elapsed);
        } else if elapsed > expected_time + Duration::from_millis(100) {
            dropped_frames += 1;
            return true;
        }

        let grey_frame = convert::resize_and_greyscale(
            frame.width,
            frame.height,
            frame.data,
            target_width,
            target_height,
        );

        if frame_count % 5 == 0 {
            current_threshold = convert::otsu_threshold(&grey_frame);
        }

        frame_count += 1;

        match mode {
            RenderMode::Otsu => {
                render::render_frame(&grey_frame, current_threshold);
            }
            RenderMode::Dither => {
                let dithered = convert::dither(&grey_frame, current_threshold);
                render::render_frame(&dithered, 128);
            }
        }

        true
    });
    eprintln!(
        "Dropped {} / {} frames ({:.1}%)",
        dropped_frames,
        total_frames,
        (dropped_frames as f64 / total_frames as f64) * 100.0
    );
}
