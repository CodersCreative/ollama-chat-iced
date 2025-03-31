use crate::{ chat::run_ollama, save::{chat::{Chat, Role},chats::{Chats, ChatsMessage, SavedChats}}, SAVE_FILE};
use crate::SideChats;

use iced::{widget::text_editor, Task};
use ollama_rs::generation::chat::ChatMessage;
use std::sync::Arc;
use crate::{ChatApp, Message};

impl ChatsMessage{
    pub fn submit(&self, id: i32, app : &mut ChatApp, new : bool) -> Task<Message>{
        let index = Chats::get_index(app, id);
        let ollama = Arc::clone(&app.logic.ollama);
        app.main_view.chats[index].loading = true;
        
        let s_index = app.main_view.chats[index].get_saved_index(app).unwrap();
        if new{
            let chat = Chat{
                role: Role::User,
                message: app.main_view.chats[index].input.text(),
                images: app.main_view.chats[index].images.clone(), 
            };
            app.main_view.chats[index].markdown.push(Chat::generate_mk(&chat.message));
            app.save.chats[s_index].0.push(chat);
        }
        

        let chat = app.save.chats[s_index].clone();
        app.main_view.chats[index].gen_chats = Arc::new(chat.get_chat_messages());
        let chat = Arc::clone(&app.main_view.chats[index].gen_chats);
        let index = app.options.get_create_model_options_index(app.main_view.chats[index].model.clone());
        //let id = id.clone();
        
        Task::perform(run_ollama(chat, app.options.0[index].clone(), ollama, app.main_view.chats[index].model.clone()), move |x| Message::Chats(ChatsMessage::Received(x), id))
    }

    pub fn received(&self, app : &mut ChatApp, id: i32, result : ChatMessage) -> Task<Message>{
        let index = Chats::get_index(app, id);
        app.main_view.chats[index].loading = false;
        let s_index = app.main_view.chats[index].get_saved_index(app);
        
        let images = match result.images{
            Some(_) => Vec::new(),
            None => Vec::new(),
        };

        let chat = Chat{
            role: Role::AI,
            message: result.content.trim().to_string(),
            images, 
        };

        match s_index{
            Some(x) => {
                app.main_view.chats[index].markdown.push(Chat::generate_mk(&chat.message));
                app.save.chats[x].0.push(chat);
            },
            None => {
                app.save.chats.push(SavedChats::new_with_chats(vec![chat]))
            },
        }

        app.main_view.chats[index].input = text_editor::Content::new();
        app.main_view.chats[index].images = Vec::new();
        app.save.save(SAVE_FILE);
        app.main_view.side_chats = SideChats::new(app.save.get_chat_previews());

        Task::none()
    }
}
