#![allow(dead_code)]
use std::{
    error::Error, fs, path::PathBuf, sync::OnceLock, time::SystemTime
};
use sqlx::{
    pool::PoolConnection, Acquire, Pool, Sqlite, SqlitePool
};

use crate::{common::Id, save::chat::{Chat, ChatBuilder, Role}, utils::{get_path_dir, get_path_settings}, Message};


pub type Conn = PoolConnection<Sqlite>;

static DB: OnceLock<Pool<Sqlite>> = OnceLock::new();


pub async fn setup() -> Result<(), Box<dyn Error>> {
    // let path = get_path_settings("db.db".to_string());
    // let path = format!("sqlite:{}", path);
    let path = "sqlite:ochat.db".to_string();
    let pool = SqlitePool::connect(path.as_str()).await?;
    // sqlx::migrate!()
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    DB.set(pool).unwrap();

    Ok(())
}


pub async fn get_conn() -> Conn {
    DB.get()
        .expect("forgot to setup DB")
        .acquire()
        .await
        .expect("db conn failed")
}

pub fn sqlite_pool() -> &'static SqlitePool {
    DB.get().expect("forgot to setup() db")
}

pub async fn get_messages(
    chat: &Id,
) -> Vec<Chat> {
    sqlx::query!(
        r#"
SELECT role, content, timestamp, images, tools 
FROM message
WHERE ("id" = ?1)
ORDER BY timestamp ASC;
    "#,
        chat.to_string(),
    )
    .fetch_all(sqlite_pool())
    .await
    .unwrap()
    .into_iter()
    .map(|r| {
        let images = serde_json::from_str(r.images).unwrap_or(Vec::new());
        let tools = serde_json::from_str(r.tools).unwrap_or(Vec::new());
        ChatBuilder::default().content(r.content).role(Role::from(r.role as usize)).timestamp(r.timestamp).images(images).tools(tools).build()
    })
    .collect()
}

pub async fn add_text_message(id : Id, message : Chat) {
    let mut conn = get_conn().await;

    sqlx::query!(
        r#"
INSERT INTO message ("id", role, content, timestamp, images, tools)
VALUES (?1, ?2, ?3, ?4, ?5, ?6); 
    "#,
        id.to_string(),
        message.role.into::<usize>(),
        message.content,
        message.timestamp,
        serde_json::to_string(message.images).unwrap_or(String::new()),
        serde_json::to_string(message.tools).unwrap_or(String::new()),
    )
    .execute(&mut *conn)
    .await
    .unwrap();
}
