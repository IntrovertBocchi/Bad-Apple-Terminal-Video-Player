use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::FfmpegEvent;
use ffmpeg_sidecar::event::OutputVideoFrame;

pub fn decode_first_frame<F: FnMut(OutputVideoFrame) -> bool>(video_path: &str,mut on_frame: F) {
    let mut ffmpeg = FfmpegCommand::new()
        .input(video_path)
        .rawvideo()
        .spawn()
        .expect("Failed to spawn ffmpeg - is it on your PATH?");
    
    for event in ffmpeg.iter().expect("failed to iterate ffmpeg output") {
        if let FfmpegEvent::OutputFrame(frame) = event {
            let should_continue = on_frame(frame);
            if !should_continue {
                break;
            }
        }
    }
}
pub fn decode_all_frames<F: FnMut(OutputVideoFrame) -> bool>(video_path: &str, mut on_frame: F) {
    let mut ffmpeg = FfmpegCommand::new()
        .input(video_path)
        .rawvideo()
        .spawn()
        .expect("failed to spawn ffmpeg - is it on your PATH?");
    
    for event in ffmpeg.iter().expect("failed to iterate ffmpeg output") {
        if let FfmpegEvent::OutputFrame(frame) = event {
            let should_continue = on_frame(frame);
            if !should_continue {
                break;
            }
        }
    }
}