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
enum GlyphStyle {
    Half,
    Quad,
    Sextant,
    Octant,
}
#[derive(Clone, Copy)]
enum RenderMode {
    Otsu,
    Dither,
    Colour,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <path-to-video> <path-to-audio> [otsu|dither]", args[0]);
        return;
    }
    let video_path = args[1].clone();
    let audio_path = args[2].clone();

    let (term_width, term_height) = terminal::size().expect("failed to read terminal size");

    let mode = match args.get(3).map(|s| s.as_str()) {
        Some("otsu") => RenderMode::Otsu,
        Some("dither") => RenderMode::Dither,
        Some("colour") | Some("color") => RenderMode::Colour,
        Some(other) => {
            eprintln!("Unknown mode '{}', defaulting to dither", other);
            RenderMode::Dither
        }
        None => RenderMode::Dither,
    };

    let style = match args.get(4).map(|s| s.as_str()) {
        Some("quad") => GlyphStyle::Quad,
        Some("sextant") => GlyphStyle::Sextant,
        Some("octant") => GlyphStyle::Octant,
        Some("half") => GlyphStyle::Half,
        Some(other) => {
            eprintln!("Unknown style '{}', defaulting to half", other);
            GlyphStyle::Half
        }
        None => GlyphStyle::Half,
    };

    let (target_width, target_height) = match style {
        GlyphStyle::Half => (term_width as u32, (term_height as u32) * 2),
        GlyphStyle::Quad => ((term_width as u32) * 2, (term_height as u32) * 2),
        GlyphStyle::Sextant => ((term_width as u32) * 2, (term_height as u32) * 3),
        GlyphStyle::Octant => ((term_width as u32) * 2, (term_height as u32) * 4),
    };

    thread::spawn(move || {
        audio::play_audio_blocking(&audio_path);
    });

    let _guard = TerminalGuard::new();

    let start_time = Instant::now();
    let mut frame_count: u32 = 0;
    let mut current_threshold: u8 = 128;
    let mut dropped_frames: u32 = 0;
    let mut total_frames: u32 = 0;
    let mut resize_time_total = Duration::from_secs(0);
    let mut render_time_total = Duration::from_secs(0);
    let mut color_frame_count: u32 = 0;

    
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

        match (mode, style) {
            (RenderMode::Otsu, GlyphStyle::Half) => {
                let grey_frame = convert::resize_and_greyscale (
                    frame.width, frame.height, frame.data.clone(),
                    target_width, target_height,
                );
                if frame_count % 5 == 0 {
                    current_threshold = convert::otsu_threshold(&grey_frame);
                }
                render::render_frame(&grey_frame, current_threshold);
            }

            (RenderMode::Otsu, GlyphStyle::Quad) => {
                let grey_frame = convert::resize_and_greyscale(
                    frame.width, frame.height, frame.data.clone(),
                    target_width, target_height,
                );
                if frame_count % 5 == 0 {
                    current_threshold = convert::otsu_threshold(&grey_frame);
                }
                render::render_frame_quadrant(&grey_frame, current_threshold);
            }

            (RenderMode::Otsu, GlyphStyle::Sextant) => {
                let grey_frame = convert::resize_and_greyscale(
                    frame.width, frame.height, frame.data.clone(),
                    target_width, target_height,
                );
                if frame_count % 5 == 0 {
                    current_threshold = convert::otsu_threshold(&grey_frame);
                }
                render::render_frame_sextant(&grey_frame, current_threshold);
            }

            (RenderMode::Otsu, GlyphStyle::Octant) => {
                let grey_frame = convert::resize_and_greyscale(
                    frame.width, frame.height, frame.data.clone(),
                    target_width, target_height,
                );
                if frame_count % 5 == 0 {
                    current_threshold = convert::otsu_threshold(&grey_frame);
                }
                render::render_frame_octant(&grey_frame, current_threshold);
            }

            (RenderMode::Dither, GlyphStyle::Half )=> {
                let grey_frame = convert::resize_and_greyscale(
                    frame.width, frame.height, frame.data.clone(),
                    target_width, target_height,
                );

                if frame_count % 5 == 0 {
                    current_threshold = convert::otsu_threshold(&grey_frame);
                }

                let dithered = convert::dither(&grey_frame, current_threshold);
                render::render_frame(&dithered, 128);
            }

            (RenderMode::Dither, GlyphStyle::Quad) => {
                let grey_frame = convert::resize_and_greyscale(
                    frame.width, frame.height, frame.data.clone(),
                    target_width, target_height,
                );
                if frame_count % 5 == 0 {
                    current_threshold = convert::otsu_threshold(&grey_frame);
                }
                let dithered = convert::dither(&grey_frame, current_threshold);
                render::render_frame_quadrant(&dithered, 128);
            }

            (RenderMode::Dither, GlyphStyle::Sextant) => {
                let grey_frame = convert::resize_and_greyscale(
                    frame.width, frame.height, frame.data.clone(),
                    target_width, target_height,
                );
                if frame_count % 5 == 0 {
                    current_threshold = convert::otsu_threshold(&grey_frame);
                }
                let dithered = convert::dither(&grey_frame, current_threshold);
                render::render_frame_sextant(&dithered, 128);
            }

            (RenderMode::Dither, GlyphStyle::Octant) => {
                let grey_frame = convert::resize_and_greyscale(
                    frame.width, frame.height, frame.data.clone(),
                    target_width, target_height,
                );
                if frame_count % 5 == 0 {
                    current_threshold = convert::otsu_threshold(&grey_frame);
                }
                let dithered = convert::dither(&grey_frame, current_threshold);
                render::render_frame_octant(&dithered, 128);
            }

            (RenderMode::Colour, GlyphStyle::Half)=> {
                let color_frame = convert::resize_color(
                    frame.width, frame.height, frame.data.clone(),
                    target_width, target_height,
                );
                let quantized_frame = convert::quantize_color(&color_frame, 32);
                render::render_frame_color(&quantized_frame);
            }

            (RenderMode::Colour, GlyphStyle::Quad) => {
                let color_frame = convert::resize_color(
                    frame.width, frame.height, frame.data.clone(),
                    target_width, target_height,
                );
                render::render_frame_quadrant_color(&color_frame);
            }

            (RenderMode::Colour, GlyphStyle::Sextant) => {
                let color_frame = convert::resize_color(
                    frame.width, frame.height, frame.data.clone(),
                    target_width, target_height,
                );
                render::render_frame_sextant_color(&color_frame);
            }

            (RenderMode::Colour, GlyphStyle::Octant) => {
                let color_frame = convert::resize_color(
                    frame.width, frame.height, frame.data.clone(),
                    target_width, target_height,
                );
                render::render_frame_octant_color(&color_frame);
            }
        }
        
        frame_count += 1;

        true
    });

    let avg_resize_ms = if color_frame_count > 0 {
        resize_time_total.as_secs_f64() * 1000.0 / color_frame_count as f64
    } else {
        0.0
    };

    let avg_render_ms = if color_frame_count > 0 {
        render_time_total.as_secs_f64() * 1000.0 / color_frame_count as f64
    }
     else {
        0.0
    };

    let stats = format!(
        "Dropped {} / {} frames ({:.1}%)\nAvg resize: {:.2} ms | Avg render/flush: {:.2} ms | Total: {:.2} ms (budget: 33.3 ms)\n",
        dropped_frames,
        total_frames,
        (dropped_frames as f64 / total_frames as f64) * 100.0,
        avg_resize_ms,
        avg_render_ms,
        avg_resize_ms + avg_render_ms
    );
    std::fs::write("playback_stats.log", stats).expect("failed to write stats file");
}
