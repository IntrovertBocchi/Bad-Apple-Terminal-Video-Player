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