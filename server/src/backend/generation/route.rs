use axum::{Router, routing::get};

use crate::backend::generation;

pub fn routes() -> Router {
    let router = Router::new()
        .route("/generation/text/run/", get(generation::text::run))
        .route("/generation/text/stream/", get(generation::text::stream));

    #[cfg(feature = "sound")]
    let router = router
        .route("/generation/speech/tts/run/", get(generation::tts::run))
        .route(
            "/generation/speech/tts/stream/",
            get(generation::tts::stream::run),
        );

    // TODO create stt endpoints
    #[cfg(feature = "voice")]
    let router = router.route("/generation/speech/stt/run/", get(generation::text::run));

    router
}
