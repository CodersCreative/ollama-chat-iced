use std::{path::PathBuf, str::FromStr};

use rusqlite::{params, Connection, Result as Rusult};
use uuid::Uuid;

use crate::{
    chats::{
        chat::{ChatBuilder, Role},
        tree::{ChatNode, ChatTree, Reason},
    },
    common::Id,
    utils::get_path_settings,
};

pub fn new_conn() -> Rusult<Connection> {
    let conn = Connection::open(get_path_settings("ochat.db".to_string()))?;

    conn.execute(
        "create table if not exists chats (
             id text primary key not null unique,
             root_id text not null unique references messages(id)
         )",
        [],
    )?;

    conn.execute(
        "create table if not exists messages (
             id text primary key not null unique,
             parent_id text not null references messages(id),
             position integer not null,
             role integer not null,
             content text not null,
             unix integer,
             reason text
         )",
        [],
    )?;

    conn.execute(
        "create table if not exists photos (
             messages_id text not null references messages(id),
             path text not null
         )",
        [],
    )?;

    Ok(conn)
}

pub fn get_tree(conn: Connection, chat_id: Id) -> Rusult<ChatTree> {
    let mut stmt = conn.prepare("select root_id from chats where id = (?1) limit 1;")?;
    let root: String = stmt
        .query_map([chat_id.0.to_string()], |row| Ok(row.get(0)?))?
        .nth(0)
        .unwrap()?;

    let mut stmt =
        conn.prepare("select * from messages where parent_id = (?1) order by position asc;")?;
    let mut stmt_images = conn.prepare("select * from photos where messages_id = (?1);")?;
    let mut id = root;
    let mut tree = ChatTree::new(ChatBuilder::default().build().unwrap());

    loop {
        if let Some(last) = tree.get_last_mut() {
            let rows = stmt.query_map([id.clone()], |row| {
                Ok(ChatNode {
                    id: Id::from_str(&row.get::<_, String>(0)?).unwrap(),
                    selected_child_index: None,
                    reason: Reason::from_index(row.get(6)?),
                    chat: ChatBuilder::default()
                        .content(row.get(4)?)
                        .role(Role::from(row.get::<_, usize>(3)?))
                        .build()
                        .unwrap(),
                    children: Vec::new(),
                })
            })?;

            for row in rows {
                if let Ok(mut row) = row {
                    let images = stmt_images.query_map([row.id.to_string()], |image| {
                        Ok(PathBuf::from(image.get::<_, String>(1)?))
                    })?;

                    row.chat.set_images(
                        images
                            .into_iter()
                            .filter(|x| x.is_ok())
                            .map(|x| x.unwrap())
                            .collect(),
                    );

                    last.children.push(row);
                }
            }

            if let Some(first) = last.children.first() {
                id = first.id.to_string();
            }
        } else {
            break;
        }
    }

    Ok(tree)
}
