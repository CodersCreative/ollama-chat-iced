use crate::backend::{CONN, errors::ServerError, utils::get_count};
use axum::{Json, extract::Path};
use ochat_types::chats::relationships::{MessageRelationship, MessageRelationshipData};

pub const RELATIONSHIP_TABLE: &str = "relationships";

pub async fn define_message_relationships() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMAFULL PERMISSIONS FULL;
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

pub async fn get_count_of_children(Path(parent): Path<String>) -> Result<Json<u8>, ServerError> {
    get_count(&format!(
        "
                SELECT count() FROM {0} WHERE parent = '{1}' GROUP ALL;
            ",
        RELATIONSHIP_TABLE,
        parent.trim(),
    ))
    .await
    .map(|x| Json(x))
}
pub async fn create_message_relationship(
    Json(mut relationship): Json<MessageRelationshipData>,
) -> Result<Json<Option<MessageRelationship>>, ServerError> {
    if relationship.index.is_none() {
        relationship.index = Some(
            get_count_of_children(Path(relationship.parent.to_string()))
                .await?
                .0,
        )
    }

    Ok(Json(
        CONN.create(RELATIONSHIP_TABLE)
            .content(relationship)
            .await?,
    ))
}

pub async fn get_message_relationship(
    id: Path<String>,
) -> Result<Json<Option<MessageRelationship>>, ServerError> {
    Ok(Json(CONN.select((RELATIONSHIP_TABLE, id.trim())).await?))
}

pub async fn update_message_relationship(
    id: Path<String>,
    Json(mut relationship): Json<MessageRelationshipData>,
) -> Result<Json<Option<MessageRelationship>>, ServerError> {
    if relationship.index.is_none() {
        let previous: Option<MessageRelationship> =
            CONN.select((RELATIONSHIP_TABLE, id.trim())).await?;
        relationship.index = Some(previous.unwrap().index)
    }

    Ok(Json(
        CONN.update((RELATIONSHIP_TABLE, id.trim()))
            .content(relationship)
            .await?,
    ))
}

pub async fn delete_message_relationship(
    id: Path<String>,
) -> Result<Json<Option<MessageRelationship>>, ServerError> {
    Ok(Json(CONN.delete((RELATIONSHIP_TABLE, id.trim())).await?))
}

pub async fn list_all_message_relationships_from_parent(
    parent: Path<String>,
) -> Result<Json<Vec<MessageRelationship>>, ServerError> {
    Ok(Json(
        CONN.query(&format!(
            "
                SELECT * FROM {0} WHERE parent = '{1}';
            ",
            RELATIONSHIP_TABLE,
            parent.trim(),
        ))
        .await?
        .take(0)?,
    ))
}

pub async fn list_all_message_relationships() -> Result<Json<Vec<MessageRelationship>>, ServerError>
{
    Ok(Json(CONN.select(RELATIONSHIP_TABLE).await?))
}
