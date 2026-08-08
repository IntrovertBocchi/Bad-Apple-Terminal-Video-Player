use badapple_player::decode;
use badapple_player::convert;
use badapple_player::render;

use std::env;
use crossterm::terminal;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --example test_dither -- <path-to-video>");
        return;
    }
    let video_path = &args[1];

    let (term_width, term_height) = terminal::size().expect("failed to read terminal size");
    let target_width = term_width as u32;
    let target_height = (term_height as u32) * 2;

    decode::decode_first_frame(video_path, |frame| {
        let grey_frame = convert::resize_and_greyscale(
            frame.width,
            frame.height,
            frame.data,
            target_width,
            target_height,
        );

        let threshold = convert::otsu_threshold(&grey_frame);
        let dithered = convert::dither(&grey_frame, threshold);
        render::render_frame(&dithered, 128);

        false
    });
}