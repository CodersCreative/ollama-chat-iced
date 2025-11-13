use axum::{Json, extract::Path};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::RecordId;

use crate::{CONN, errors::ServerError};

const RELATIONSHIP_TABLE: &str = "relationships";

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct MessageRelationshipData {
    pub parent: String,
    pub child: String,
    #[builder(default = "None")]
    pub reason: Option<Reason>,
    #[builder(default = "None")]
    pub index: Option<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageRelationship {
    pub parent: String,
    pub child: String,
    pub reason: Option<Reason>,
    pub index: u8,
    id: RecordId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Reason {
    Model,
    Regeneration,
    Sibling,
}

pub async fn define_message_relationships() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS parent ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS child ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS reason ON TABLE {0} TYPE option<string>;
DEFINE FIELD IF NOT EXISTS index ON TABLE {0} TYPE int;
",
            RELATIONSHIP_TABLE,
        ))
        .await?;
    Ok(())
}

pub async fn create_message_relationship(
    Json(mut relationship): Json<MessageRelationshipData>,
) -> Result<Json<Option<MessageRelationship>>, ServerError> {
    if relationship.index.is_none() {
        let mut count: u8 = 0;
        let query: Option<Value> = CONN
            .query(&format!(
                "
                SELECT count() FROM {0} WHERE parent = '{1}' GROUP ALL;
            ",
                RELATIONSHIP_TABLE, relationship.parent,
            ))
            .await?
            .take(0)?;

        if let Some(mut query) = query {
            if query.is_array() {
                query = query[0].clone();
            }
            if query.is_object() {
                count = query["count"].as_number().unwrap().as_u64().unwrap() as u8;
            }
        }

        relationship.index = Some(count)
    }

    let relationship = CONN
        .create(RELATIONSHIP_TABLE)
        .content(relationship)
        .await?;

    Ok(Json(relationship))
}

pub async fn get_message_relationship(
    id: Path<String>,
) -> Result<Json<Option<MessageRelationship>>, ServerError> {
    let relationship = CONN.select((RELATIONSHIP_TABLE, &*id)).await?;
    Ok(Json(relationship))
}

pub async fn update_message_relationship(
    id: Path<String>,
    Json(mut relationship): Json<MessageRelationshipData>,
) -> Result<Json<Option<MessageRelationship>>, ServerError> {
    if relationship.index.is_none() {
        let previous: Option<MessageRelationship> = CONN.select((RELATIONSHIP_TABLE, &*id)).await?;
        relationship.index = Some(previous.unwrap().index)
    }

    let relationship = CONN
        .update((RELATIONSHIP_TABLE, &*id))
        .content(relationship)
        .await?;

    Ok(Json(relationship))
}

pub async fn delete_message_relationship(
    id: Path<String>,
) -> Result<Json<Option<MessageRelationship>>, ServerError> {
    let relationship = CONN.delete((RELATIONSHIP_TABLE, &*id)).await?;
    Ok(Json(relationship))
}

pub async fn list_all_message_relationships_from_parent(
    parent: Path<String>,
) -> Result<Json<Vec<MessageRelationship>>, ServerError> {
    let query: Vec<MessageRelationship> = CONN
        .query(&format!(
            "
                SELECT * FROM {0} WHERE parent = '{1}';
            ",
            RELATIONSHIP_TABLE, &*parent,
        ))
        .await?
        .take(0)?;

    Ok(Json(query))
}

pub async fn list_all_message_relationships() -> Result<Json<Vec<MessageRelationship>>, ServerError>
{
    let relationship = CONN.select(RELATIONSHIP_TABLE).await?;
    Ok(Json(relationship))
}
