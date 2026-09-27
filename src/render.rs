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

pub fn render_frame_color(rgb_frame: &image::RgbImage) {
    let (width, height) = rgb_frame.dimensions();
    let mut stdout = stdout();

    queue!(stdout, MoveTo(0, 0)).expect("failed to move cursor");

    let mut last_bg: Option<Color> = None;
    let mut last_fg: Option<Color> = None;

    let mut y = 0;
    while y + 1 < height {
        for x in 0..width {
            let top_pixel = rgb_frame.get_pixel(x, y);
            let bottom_pixel = rgb_frame.get_pixel(x, y + 1);

            let top_color = Color::Rgb { r: top_pixel[0], g: top_pixel[1], b: top_pixel[2] };
            let bottom_color = Color::Rgb { r: bottom_pixel[0], g: bottom_pixel[1], b: bottom_pixel[2] };

            if last_bg != Some(top_color) {
                queue!(stdout, SetBackgroundColor(top_color)).expect("failed to queue background colour");
                last_bg = Some(top_color);
            }

            if last_fg != Some(bottom_color) {
                queue!(stdout, SetForegroundColor(bottom_color)).expect("failed to queue foreground colour");
                last_fg = Some(bottom_color);
            }

            queue!(stdout, Print("▀")).expect("failed to queue character");
        }
        queue!(stdout, Print("\r\n")).expect("failed to queue newline");
        y += 2;
    }

    stdout.flush().expect("failed to flush stdout");
}

// Render quadrants for glyph mapping
fn quadrant_glyph(pattern: u8) -> char {
    match pattern {
        0b0000 => ' ',
        0b0001 => '▘', // TL
        0b0010 => '▝', // TR
        0b0011 => '▀', // TL+TR
        0b0100 => '▖', // BL
        0b0101 => '▌', // TL+BL
        0b0110 => '▞', // TR+BL
        0b0111 => '▛', // TL+TR+BL
        0b1000 => '▗', // BR
        0b1001 => '▚', // TL+BR
        0b1010 => '▐', // TR+BR
        0b1011 => '▜', // TL+TR+BR
        0b1100 => '▄', // BL+BR
        0b1101 => '▙', // TL+BL+BR
        0b1110 => '▟', // TR+BL+BR
        0b1111 => '█', // all four
        _ => unreachable!(),
    }
}

// Otsu / Dither - Binary quadrant rendering
pub fn render_frame_quadrant(image_frame: &image::GrayImage, threshold: u8) {
    let (width, height) = image_frame.dimensions();
    let mut stdout = stdout();
    queue!(stdout, MoveTo(0, 0)).expect("failed to move cursor");

    let mut last_bg: Option<Color> = None;
    let mut last_fg: Option<Color> = None;

    let mut y = 0;
    while y + 1 < height {
        let mut x = 0;
        while x + 1 < width {
            let tl = image_frame.get_pixel(x, y)[0] > threshold;
            let tr = image_frame.get_pixel(x + 1, y)[0] > threshold;
            let bl = image_frame.get_pixel(x, y + 1)[0] > threshold;
            let br = image_frame.get_pixel(x + 1, y + 1)[0] > threshold;

            let pattern: u8 =
                (tl as u8) | ((tr as u8) << 1) | ((bl as u8) << 2) | ((br as u8) << 3);
            let glyph = quadrant_glyph(pattern);

            if last_fg != Some(Color::White) {
                queue!(stdout, SetForegroundColor(Color::White)).expect("failed to queue fg");
                last_fg = Some(Color::White);
            }
            if last_bg != Some(Color::Black) {
                queue!(stdout, SetBackgroundColor(Color::Black)).expect("failed to queue bg");
                last_bg = Some(Color::Black);
            }

            queue!(stdout, Print(glyph)).expect("failed to queue glyph");
            x += 2;
        }
        queue!(stdout, Print("\r\n")).expect("failed to queue newline");
        y += 2;
    }

    stdout.flush().expect("failed to flush stdout");
}

// Render frame for colour 
pub fn render_frame_quadrant_color(rgb_frame: &image::RgbImage) {
    let (width, height) = rgb_frame.dimensions();
    let mut stdout = stdout();
    queue!(stdout, MoveTo(0, 0)).expect("failed to move cursor");

    let mut last_bg: Option<Color> = None;
    let mut last_fg: Option<Color> = None;

    let luma = |p: &image::Rgb<u8>| -> f32 {
        0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32
    };

    let mut y = 0;
    while y + 1 < height {
        let mut x = 0;
        while x + 1 < width {
            let px = [
                rgb_frame.get_pixel(x, y),
                rgb_frame.get_pixel(x + 1, y),
                rgb_frame.get_pixel(x, y + 1),
                rgb_frame.get_pixel(x + 1, y + 1),
            ];
            let luminances: [f32; 4] = [luma(px[0]), luma(px[1]), luma(px[2]), luma(px[3])];
            let block_avg: f32 = luminances.iter().sum::<f32>() / 4.0;

            let mut pattern: u8 = 0;
            let mut light_sum = [0u32; 3];
            let mut light_count = 0u32;
            let mut dark_sum = [0u32; 3];
            let mut dark_count = 0u32;

            for i in 0..4 {
                let p = px[i];
                if luminances[i] > block_avg {
                    pattern |= 1 << i;
                    light_sum[0] += p[0] as u32;
                    light_sum[1] += p[1] as u32;
                    light_sum[2] += p[2] as u32;
                    light_count += 1;
                } else {
                    dark_sum[0] += p[0] as u32;
                    dark_sum[1] += p[1] as u32;
                    dark_sum[2] += p[2] as u32;
                    dark_count += 1;
                }
            }

            let avg_color = |sum: [u32; 3], count: u32| -> Color {
                if count == 0 {
                    Color::Rgb { r: 0, g: 0, b: 0 }
                } else {
                    Color::Rgb {
                        r: (sum[0] / count) as u8,
                        g: (sum[1] / count) as u8,
                        b: (sum[2] / count) as u8,
                    }
                }
            };

            let fg_color = avg_color(light_sum, light_count);
            let bg_color = avg_color(dark_sum, dark_count);
            let glyph = quadrant_glyph(pattern);

            if last_fg != Some(fg_color) {
                queue!(stdout, SetForegroundColor(fg_color)).expect("failed to queue fg");
                last_fg = Some(fg_color);
            }
            if last_bg != Some(bg_color) {
                queue!(stdout, SetBackgroundColor(bg_color)).expect("failed to queue bg");
                last_bg = Some(bg_color);
            }

            queue!(stdout, Print(glyph)).expect("failed to queue glyph");
            x += 2;
        }
        queue!(stdout, Print("\r\n")).expect("failed to queue newline");
        y += 2;
    }

    stdout.flush().expect("failed to flush stdout");
}

// Sextant glyphs
const SEXTANTS: [char; 64] = [
    ' ', '🬀', '🬁', '🬂', '🬃', '🬄', '🬅', '🬆', '🬇', '🬈', '🬉', '🬊', '🬋', '🬌', '🬍', '🬎', '🬏', '🬐', '🬑',
    '🬒', '🬓', '▌', '🬔', '🬕', '🬖', '🬗', '🬘', '🬙', '🬚', '🬛', '🬜', '🬝', '🬞', '🬟', '🬠', '🬡', '🬢', '🬣',
    '🬤', '🬥', '🬦', '🬧', '▐', '🬨', '🬩', '🬪', '🬫', '🬬', '🬭', '🬮', '🬯', '🬰', '🬱', '🬲', '🬳', '🬴', '🬵',
    '🬶', '🬷', '🬸', '🬹', '🬺', '🬻', '█',
];

fn sextant_glyph(pattern: u8) -> char {
    SEXTANTS[pattern as usize]
}

// Otsu / Dither Sextant Rendering
pub fn render_frame_sextant(image_frame: &image::GrayImage, threshold: u8) {
    let (width, height) = image_frame.dimensions();
    let mut stdout = stdout();
    queue!(stdout, MoveTo(0, 0)).expect("failed to move cursor");

    let mut last_bg: Option<Color> = None;
    let mut last_fg: Option<Color> = None;

    let mut y = 0;
    while y + 2 < height {
        let mut x = 0;
        while x + 1 < width {
            let px_on = |dx: u32, dy: u32| image_frame.get_pixel(x + dx, y + dy)[0] > threshold;

            let bits = [
                px_on(0, 0), px_on(1, 0), // top row
                px_on(0, 1), px_on(1, 1), // middle row
                px_on(0, 2), px_on(1, 2), // bottom row
            ];

            let mut pattern: u8 = 0;
            for (i, &on) in bits.iter().enumerate() {
                if on {
                    pattern |= 1 << i;
                }
            }
            let glyph = sextant_glyph(pattern);

            if last_fg != Some(Color::White) {
                queue!(stdout, SetForegroundColor(Color::White)).expect("failed to queue fg");
                last_fg = Some(Color::White);
            }
            if last_bg != Some(Color::Black) {
                queue!(stdout, SetBackgroundColor(Color::Black)).expect("failed to queue bg");
                last_bg = Some(Color::Black);
            }

            queue!(stdout, Print(glyph)).expect("failed to queue glyph");
            x += 2;
        }
        queue!(stdout, Print("\r\n")).expect("failed to queue newline");
        y += 3;
    }

    stdout.flush().expect("failed to flush stdout");
}

// Colour sextant rendering
pub fn render_frame_sextant_color(rgb_frame: &image::RgbImage) {
    let (width, height) = rgb_frame.dimensions();
    let mut stdout = stdout();
    queue!(stdout, MoveTo(0, 0)).expect("failed to move cursor");

    let mut last_bg: Option<Color> = None;
    let mut last_fg: Option<Color> = None;

    let luma = |p: &image::Rgb<u8>| -> f32 {
        0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32
    };

    let mut y = 0;
    while y + 2 < height {
        let mut x = 0;
        while x + 1 < width {
            let px = [
                rgb_frame.get_pixel(x, y),     rgb_frame.get_pixel(x + 1, y),
                rgb_frame.get_pixel(x, y + 1), rgb_frame.get_pixel(x + 1, y + 1),
                rgb_frame.get_pixel(x, y + 2), rgb_frame.get_pixel(x + 1, y + 2),
            ];
            let luminances: Vec<f32> = px.iter().map(|p| luma(p)).collect();
            let block_avg: f32 = luminances.iter().sum::<f32>() / luminances.len() as f32;

            let mut pattern: u8 = 0;
            let mut light_sum = [0u32; 3];
            let mut light_count = 0u32;
            let mut dark_sum = [0u32; 3];
            let mut dark_count = 0u32;

            for i in 0..6 {
                let p = px[i];
                if luminances[i] > block_avg {
                    pattern |= 1 << i;
                    light_sum[0] += p[0] as u32;
                    light_sum[1] += p[1] as u32;
                    light_sum[2] += p[2] as u32;
                    light_count += 1;
                } else {
                    dark_sum[0] += p[0] as u32;
                    dark_sum[1] += p[1] as u32;
                    dark_sum[2] += p[2] as u32;
                    dark_count += 1;
                }
            }

            let avg_color = |sum: [u32; 3], count: u32| -> Color {
                if count == 0 {
                    Color::Rgb { r: 0, g: 0, b: 0 }
                } else {
                    Color::Rgb {
                        r: (sum[0] / count) as u8,
                        g: (sum[1] / count) as u8,
                        b: (sum[2] / count) as u8,
                    }
                }
            };

            let fg_color = avg_color(light_sum, light_count);
            let bg_color = avg_color(dark_sum, dark_count);
            let glyph = sextant_glyph(pattern);

            if last_fg != Some(fg_color) {
                queue!(stdout, SetForegroundColor(fg_color)).expect("failed to queue fg");
                last_fg = Some(fg_color);
            }
            if last_bg != Some(bg_color) {
                queue!(stdout, SetBackgroundColor(bg_color)).expect("failed to queue bg");
                last_bg = Some(bg_color);
            }

            queue!(stdout, Print(glyph)).expect("failed to queue glyph");
            x += 2;
        }
        queue!(stdout, Print("\r\n")).expect("failed to queue newline");
        y += 3;
    }

    stdout.flush().expect("failed to flush stdout");
}

// Octant Glyphs
const OCTANTS: [char; 256] = [
    ' ', '𜺨', '𜺫', '🮂', '𜴀', '▘', '𜴁', '𜴂', '𜴃', '𜴄', '▝', '𜴅', '𜴆', '𜴇', '𜴈', '▀', '𜴉', '𜴊', '𜴋',
    '𜴌', '🯦', '𜴍', '𜴎', '𜴏', '𜴐', '𜴑', '𜴒', '𜴓', '𜴔', '𜴕', '𜴖', '𜴗', '𜴘', '𜴙', '𜴚', '𜴛', '𜴜', '𜴝',
    '𜴞', '𜴟', '🯧', '𜴠', '𜴡', '𜴢', '𜴣', '𜴤', '𜴥', '𜴦', '𜴧', '𜴨', '𜴩', '𜴪', '𜴫', '𜴬', '𜴭', '𜴮', '𜴯',
    '𜴰', '𜴱', '𜴲', '𜴳', '𜴴', '𜴵', '🮅', '𜺣', '𜴶', '𜴷', '𜴸', '𜴹', '𜴺', '𜴻', '𜴼', '𜴽', '𜴾', '𜴿', '𜵀',
    '𜵁', '𜵂', '𜵃', '𜵄', '▖', '𜵅', '𜵆', '𜵇', '𜵈', '▌', '𜵉', '𜵊', '𜵋', '𜵌', '▞', '𜵍', '𜵎', '𜵏', '𜵐',
    '▛', '𜵑', '𜵒', '𜵓', '𜵔', '𜵕', '𜵖', '𜵗', '𜵘', '𜵙', '𜵚', '𜵛', '𜵜', '𜵝', '𜵞', '𜵟', '𜵠', '𜵡', '𜵢',
    '𜵣', '𜵤', '𜵥', '𜵦', '𜵧', '𜵨', '𜵩', '𜵪', '𜵫', '𜵬', '𜵭', '𜵮', '𜵯', '𜵰', '𜺠', '𜵱', '𜵲', '𜵳', '𜵴',
    '𜵵', '𜵶', '𜵷', '𜵸', '𜵹', '𜵺', '𜵻', '𜵼', '𜵽', '𜵾', '𜵿', '𜶀', '𜶁', '𜶂', '𜶃', '𜶄', '𜶅', '𜶆', '𜶇',
    '𜶈', '𜶉', '𜶊', '𜶋', '𜶌', '𜶍', '𜶎', '𜶏', '▗', '𜶐', '𜶑', '𜶒', '𜶓', '▚', '𜶔', '𜶕', '𜶖', '𜶗', '▐',
    '𜶘', '𜶙', '𜶚', '𜶛', '▜', '𜶜', '𜶝', '𜶞', '𜶟', '𜶠', '𜶡', '𜶢', '𜶣', '𜶤', '𜶥', '𜶦', '𜶧', '𜶨', '𜶩',
    '𜶪', '𜶫', '▂', '𜶬', '𜶭', '𜶮', '𜶯', '𜶰', '𜶱', '𜶲', '𜶳', '𜶴', '𜶵', '𜶶', '𜶷', '𜶸', '𜶹', '𜶺', '𜶻',
    '𜶼', '𜶽', '𜶾', '𜶿', '𜷀', '𜷁', '𜷂', '𜷃', '𜷄', '𜷅', '𜷆', '𜷇', '𜷈', '𜷉', '𜷊', '𜷋', '𜷌', '𜷍', '𜷎',
    '𜷏', '𜷐', '𜷑', '𜷒', '𜷓', '𜷔', '𜷕', '𜷖', '𜷗', '𜷘', '𜷙', '𜷚', '▄', '𜷛', '𜷜', '𜷝', '𜷞', '▙', '𜷟',
    '𜷠', '𜷡', '𜷢', '▟', '𜷣', '▆', '𜷤', '𜷥', '█',
];

fn octant_glyph(pattern: u8) -> char {
    OCTANTS[pattern as usize]
}

// Binary Octant Rendering
pub fn render_frame_octant(image_frame: &image::GrayImage, threshold: u8) {
    let (width, height) = image_frame.dimensions();
    let mut stdout = stdout();
    queue!(stdout, MoveTo(0, 0)).expect("failed to move cursor");

    let mut last_bg: Option<Color> = None;
    let mut last_fg: Option<Color> = None;

    let mut y = 0;
    while y + 3 < height {
        let mut x = 0;
        while x + 1 < width {
            let px_on = |dx: u32, dy: u32| image_frame.get_pixel(x + dx, y + dy)[0] > threshold;

            let bits = [
                px_on(0, 0), px_on(1, 0),
                px_on(0, 1), px_on(1, 1),
                px_on(0, 2), px_on(1, 2),
                px_on(0, 3), px_on(1, 3),
            ];

            let mut pattern: u8 = 0;
            for (i, &on) in bits.iter().enumerate() {
                if on {
                    pattern |= 1 << i;
                }
            }
            let glyph = octant_glyph(pattern);

            if last_fg != Some(Color::White) {
                queue!(stdout, SetForegroundColor(Color::White)).expect("failed to queue fg");
                last_fg = Some(Color::White);
            }
            if last_bg != Some(Color::Black) {
                queue!(stdout, SetBackgroundColor(Color::Black)).expect("failed to queue bg");
                last_bg = Some(Color::Black);
            }

            queue!(stdout, Print(glyph)).expect("failed to queue glyph");
            x += 2;
        }
        queue!(stdout, Print("\r\n")).expect("failed to queue newline");
        y += 4;
    }

    stdout.flush().expect("failed to flush stdout");
}

// Colour Octant Rendering
pub fn render_frame_octant_color(rgb_frame: &image::RgbImage) {
    let (width, height) = rgb_frame.dimensions();
    let mut stdout = stdout();
    queue!(stdout, MoveTo(0, 0)).expect("failed to move cursor");

    let mut last_bg: Option<Color> = None;
    let mut last_fg: Option<Color> = None;

    let luma = |p: &image::Rgb<u8>| -> f32 {
        0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32
    };

    let mut y = 0;
    while y + 3 < height {
        let mut x = 0;
        while x + 1 < width {
            let px = [
                rgb_frame.get_pixel(x, y),     rgb_frame.get_pixel(x + 1, y),
                rgb_frame.get_pixel(x, y + 1), rgb_frame.get_pixel(x + 1, y + 1),
                rgb_frame.get_pixel(x, y + 2), rgb_frame.get_pixel(x + 1, y + 2),
                rgb_frame.get_pixel(x, y + 3), rgb_frame.get_pixel(x + 1, y + 3),
            ];
            let luminances: Vec<f32> = px.iter().map(|p| luma(p)).collect();
            let block_avg: f32 = luminances.iter().sum::<f32>() / luminances.len() as f32;

            let mut pattern: u8 = 0;
            let mut light_sum = [0u32; 3];
            let mut light_count = 0u32;
            let mut dark_sum = [0u32; 3];
            let mut dark_count = 0u32;

            for i in 0..8 {
                let p = px[i];
                if luminances[i] > block_avg {
                    pattern |= 1 << i;
                    light_sum[0] += p[0] as u32;
                    light_sum[1] += p[1] as u32;
                    light_sum[2] += p[2] as u32;
                    light_count += 1;
                } else {
                    dark_sum[0] += p[0] as u32;
                    dark_sum[1] += p[1] as u32;
                    dark_sum[2] += p[2] as u32;
                    dark_count += 1;
                }
            }

            let avg_color = |sum: [u32; 3], count: u32| -> Color {
                if count == 0 {
                    Color::Rgb { r: 0, g: 0, b: 0 }
                } else {
                    Color::Rgb {
                        r: (sum[0] / count) as u8,
                        g: (sum[1] / count) as u8,
                        b: (sum[2] / count) as u8,
                    }
                }
            };

            let fg_color = avg_color(light_sum, light_count);
            let bg_color = avg_color(dark_sum, dark_count);
            let glyph = octant_glyph(pattern);

            if last_fg != Some(fg_color) {
                queue!(stdout, SetForegroundColor(fg_color)).expect("failed to queue fg");
                last_fg = Some(fg_color);
            }
            if last_bg != Some(bg_color) {
                queue!(stdout, SetBackgroundColor(bg_color)).expect("failed to queue bg");
                last_bg = Some(bg_color);
            }

            queue!(stdout, Print(glyph)).expect("failed to queue glyph");
            x += 2;
        }
        queue!(stdout, Print("\r\n")).expect("failed to queue newline");
        y += 4;
    }

    stdout.flush().expect("failed to flush stdout");
}