use iced::futures::StreamExt;
use kalosm_sound::*;
use rodio::buffer::SamplesBuffer;

pub async fn get_audio(stream: MicStream) -> Option<SamplesBuffer<f32>> {
    let x = stream
        .voice_activity_stream()
        .rechunk_voice_activity()
        .with_end_threshold(0.3)
        .into_future()
        .await;
    return x.0;
}

pub async fn transcribe(stream: Option<SamplesBuffer<f32>>) -> Result<String, String> {
    let model = Whisper::builder()
        .with_source(WhisperSource::QuantizedTiny)
        .build()
        .await
        .map_err(|e| e.to_string())?;
    let x = model.transcribe(stream.unwrap()).into_future().await;
    Ok(x.0.unwrap().text().to_string())
}
