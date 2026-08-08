use image::{DynamicImage, RgbImage, imageops::FilterType};

pub fn resize_and_greyscale(
    width: u32,
    height: u32,
    rgb_data: Vec<u8>,
    target_width: u32,
    target_height: u32,
) -> image::GrayImage {
    let rgb_image = RgbImage::from_raw(width, height, rgb_data)
        .expect("frame dimensions didn't match pixel data length");

    let dynamic_image = DynamicImage::ImageRgb8(rgb_image);

    let resized = dynamic_image.resize_exact(
        target_width,
        target_height,
        FilterType::Lanczos3,
    );

    resized.into_luma8()
}

// Otsu
pub fn otsu_threshold(grey_frame: &image::GrayImage) -> u8 {
    let mut histogram = [0u32; 256];
    for pixel in grey_frame.pixels() {
        histogram[pixel[0] as usize] += 1;
    }

    let total_pixels: u32 = histogram.iter().sum();
    let sum_all: f64 = histogram
        .iter()
        .enumerate()
        .map(|(value, &count)| value as f64 * count as f64)
        .sum();

    let mut sum_background: f64 = 0.0;
    let mut weight_background: u32 = 0;
    let mut max_variance: f64 = 0.0;
    let mut best_threshold: u8 = 128;

    for threshold in 0..256 {
        weight_background += histogram[threshold];
        if weight_background == 0 {
            continue;
        }

        let weight_foreground = total_pixels - weight_background;
        if weight_foreground == 0 {
            break;
        }

        sum_background += threshold as f64 * histogram[threshold] as f64;

        let mean_background = sum_background / weight_background as f64;
        let mean_foreground = (sum_all - sum_background) / weight_foreground as f64;

        let between_class_variance = weight_background as f64
            * weight_foreground as f64
            * (mean_background - mean_foreground).powi(2);

        if between_class_variance > max_variance {
            max_variance = between_class_variance;
            best_threshold = threshold as u8;
        }
    }
    best_threshold
}

// Floyd-Steinberg Dithering
pub fn dither(grey_frame: &image::GrayImage, threshold: u8) -> image::GrayImage {
    let (width, height) = grey_frame.dimensions();
    let mut buffer: Vec<f32> = grey_frame.pixels().map(|p| p[0] as f32).collect();
    let mut output = image::GrayImage::new(width, height);

    let idx = |x: u32, y: u32| -> usize { (y * width + x) as usize };

    for y in 0..height {
        for x in 0..width {
            let old_value = buffer[idx(x, y)];
            let new_value: f32 = if old_value > threshold as f32 { 255.0 } else { 0.0 };
            output.put_pixel(x, y, image::Luma([new_value as u8]));

            let error = old_value - new_value;

            if x + 1 < width {
                buffer[idx(x + 1, y)] += error * 7.0 / 16.0;
            }
            if x > 0 && y + 1 < height {
                buffer[idx(x - 1, y + 1)] += error * 3.0 / 16.0;
            }
            if y + 1 < height {
                buffer[idx(x, y + 1)] += error * 5.0 / 16.0;
            }
            if x + 1 < width && y + 1 < height {
                buffer[idx(x + 1, y + 1)] += error * 1.0 / 16.0;
            }
        }
    }

    output
}