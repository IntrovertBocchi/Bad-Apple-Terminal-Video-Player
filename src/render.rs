use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
};
use std::io::{stdout, Write};

pub fn render_frame(grey_frame: &image::GrayImage, threshold: u8) {
    let (width, height) = grey_frame.dimensions();
    let mut stdout = stdout();

    queue!(stdout, MoveTo(0,0)).expect("failed to move cursor");

    let mut y = 0;
    while y + 1 < height {
        for x in 0..width {
            let top_pixel = grey_frame.get_pixel(x, y)[0];
            let bottom_pixel = grey_frame.get_pixel(x, y + 1)[0];

            let top_color = if top_pixel > threshold { Color::White } else { Color::Black };
            let bottom_color = if bottom_pixel > threshold { Color::White } else { Color::Black };

            queue!(
                stdout,
                SetBackgroundColor(top_color),
                SetForegroundColor(bottom_color),
                Print("▀")
            )
            .expect("failed to queue character");
        }
        queue!(stdout, Print("\r\n")).expect("failed to queue newline");
        y += 2;
    }

    stdout.flush().expect("failed to flush stdout");
}