use rodio::{OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;

pub fn play_audio_blocking(audio_path: &str) {
    let (_stream, stream_handle) = OutputStream::try_default()
        .expect("failed to open default audio output device.");

    let sink = Sink::try_new(&stream_handle).expect("failed to create audio sink");

    let file = File::open(audio_path).expect("failed to open audio file");
    let source = rodio::Decoder::new(BufReader::new(file))
        .expect("failed to decode audio file - check format/codec");

    sink.append(source);
    sink.sleep_until_end();
}