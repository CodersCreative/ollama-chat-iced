use axum::{Json, extract::Path, http::HeaderMap};
use ochat_types::{
    surreal::RecordId,
    user::{SigninData, SignupData, Token, User},
};
use serde::{Deserialize, Serialize};
use surrealdb::opt::auth::Record;

pub mod route;

use crate::{CONN, DATABASE, NAMESPACE, errors::ServerError, set_db, utils::get_count};
const USER_TABLE: &str = "user";
const AUTH_TABLE: &str = "auth";

pub async fn define_users() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS name ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS email ON TABLE {0} TYPE string ASSERT string::is::email($value);
DEFINE FIELD IF NOT EXISTS bio ON TABLE {0} TYPE string DEFAULT '';
DEFINE FIELD IF NOT EXISTS gender ON TABLE {0} TYPE option<string>;
DEFINE FIELD IF NOT EXISTS role ON TABLE {0} TYPE string DEFAULT 'User';
DEFINE FIELD IF NOT EXISTS enabled ON TABLE {0} TYPE bool DEFAULT true;
DEFINE INDEX name_index ON TABLE {0} COLUMNS name UNIQUE;
DEFINE INDEX email_index ON TABLE {0} COLUMNS email UNIQUE;


DEFINE TABLE IF NOT EXISTS {1} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS name ON TABLE {1} TYPE string;
DEFINE FIELD IF NOT EXISTS email ON TABLE {1} TYPE string ASSERT string::is::email($value);
DEFINE FIELD IF NOT EXISTS password ON TABLE {1} TYPE string;

DEFINE ACCESS IF NOT EXISTS {1} ON DATABASE TYPE RECORD
    SIGNUP ( CREATE {1} SET name = $name, email = $email, password = crypto::argon2::generate($password) )
    SIGNIN ( SELECT * FROM {1} WHERE name = $name AND crypto::argon2::compare(password, $password) )
    DURATION FOR TOKEN 1w, FOR SESSION 1w
;
",
            USER_TABLE, AUTH_TABLE
        ))
        .await?;
    Ok(())
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenData {
    pub id: RecordId,
    pub user_id: String,
    pub exp: String,
}

pub async fn signup(data: Json<SignupData>) -> Result<Json<Token>, ServerError> {
    let jwt = CONN
        .signup(Record {
            namespace: NAMESPACE,
            database: DATABASE,
            access: AUTH_TABLE,
            params: data.0.clone(),
        })
        .await?
        .into_insecure_token();

    let _ = set_db().await?;

    let count = get_count(&format!(
        "
                SELECT count() FROM {0} GROUP ALL;
            ",
        USER_TABLE,
    ))
    .await?;

    if count <= 1 {
        let _ = CONN
            .query(&format!(
                "
                UPDATE {0} SET role = 'Admin';
            ",
                USER_TABLE,
            ))
            .await;
    }

    Ok(Json(Token::new(jwt)))
}

pub async fn signin(data: Json<SigninData>) -> Result<Json<Token>, ServerError> {
    let jwt = CONN
        .signin(Record {
            namespace: NAMESPACE,
            database: DATABASE,
            access: AUTH_TABLE,
            params: data.0.clone(),
        })
        .await?
        .into_insecure_token();

    let _ = set_db().await?;

    Ok(Json(Token::new(jwt)))
}

pub async fn authenticate(header: &HeaderMap) -> Result<(), ServerError> {
    let jwt: String = match header.get("Authorization") {
        Some(x) => x,
        _ => {
            return Err(ServerError::Unknown(String::from(
                "AUTH ERROR : Not Logged In.",
            )));
        }
    }
    .to_str()
    .map_err(|e| ServerError::Unknown(e.to_string()))?
    .to_string();

    let _ = CONN.authenticate(jwt).await?;

    Ok(())
}

pub async fn get_current_user() -> Result<Json<Option<User>>, ServerError> {
    let mut user: Vec<User> = CONN
        .query(&format!(
            "
                SELECT * FROM type::record({0}, meta::id($auth.id));
            ",
            USER_TABLE,
        ))
        .await?
        .take(0)?;

    if user.is_empty() {
        user = CONN
            .query(&format!(
                "
                CREATE type::record({0}, meta::id($auth.id)) SET name = $auth.name, email = $auth.email;
            ",
                USER_TABLE,
            ))
            .await?
            .take(0)?;

        if user.is_empty() {
            return Ok(Json(None));
        }
    }
    Ok(Json(Some(user.remove(0))))
}

pub async fn get_user_from_name(name: String) -> Result<Option<User>, ServerError> {
    let mut user: Vec<User> = CONN
        .query(&format!(
            "
                SELECT * FROM {0} WHERE name = '{1}';
            ",
            USER_TABLE,
            name.trim(),
        ))
        .await?
        .take(0)?;

    if user.is_empty() {
        return Ok(None);
    }

    Ok(Some(user.remove(0)))
}

pub async fn get_user_from_id(id: Path<String>) -> Result<Json<Option<User>>, ServerError> {
    Ok(Json(CONN.select((USER_TABLE, id.trim())).await?))
}

pub async fn list_all_users() -> Result<Json<Vec<User>>, ServerError> {
    Ok(Json(CONN.select(USER_TABLE).await?))
}
