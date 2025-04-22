use std::path::PathBuf;
use iced::Task;
use crate::{common::Id, save::chats::ChatsMessage, ChatApp, Message};

const IMAGE_FORMATS: &[&str] = &[
    "bmp", "dds", "ff", "gif", "hdr", "ico", "jpeg", "jpg", "exr", "png", "pnm", "qoi", "tga",
    "tiff", "webp",
];

impl ChatApp{
    pub fn pick_images(id : Id) -> Task<Message>{
        Task::perform(Self::load_images(), move |x| Message::Chats(ChatsMessage::PickedImage(x), id))
    }

    async fn load_images() -> Result<Vec<PathBuf>, String> {
        let files = rfd::AsyncFileDialog::new()
        .add_filter("Image", IMAGE_FORMATS)
        .pick_files()
        .await;
        
        if let Some(files) = files{
            return Ok(files.iter().map(|x| x.path().to_path_buf()).collect());
        }

        Err("Failed".to_string())
    }
}
