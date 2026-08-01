mod decode;
mod convert;
mod render;
mod audio;
mod terminal_guard;

use std::env;
use std::time::{Duration, Instant};
use std::thread;
use crossterm::terminal;
use crossterm::event::{poll, read, Event, KeyCode};
use terminal_guard::TerminalGuard;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-video>", args[0]);
        return;
    }
    let video_path = args[1].clone();
    let audio_path = args[2].clone();

    let (term_width, term_height) = terminal::size().expect("failed to read terminal size");
    let target_width = term_width as u32;
    let target_height = (term_height as u32) * 2;
    
    thread::spawn(move || {
        audio::play_audio_blocking(&audio_path);
    });

    let _guard = TerminalGuard::new();
    
    let start_time = Instant::now();

    decode::decode_all_frames(&video_path, |frame| {
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
            return true;
        }

        let grey_frame = convert::resize_and_greyscale(
            frame.width,
            frame.height,
            frame.data,
            target_width,
            target_height,
        );
        render::render_frame(&grey_frame);

        true
    });
}