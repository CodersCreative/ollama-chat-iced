use axum::{Extension, Json, extract::Path, http::HeaderMap};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use ochat_types::{
    surreal::RecordId,
    user::{SigninData, SignupData, Token, User},
};
use serde::{Deserialize, Serialize};
use surrealdb::opt::auth::Record;

pub mod route;

use crate::{CONN, DATABASE, NAMESPACE, errors::ServerError, set_db, utils::get_count};
const USER_TABLE: &str = "user";
const TOKEN_TABLE: &str = "token";

pub async fn define_users() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS name ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS email ON TABLE {0} TYPE string ASSERT string::is::email($value);
DEFINE FIELD IF NOT EXISTS email ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS password ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS bio ON TABLE {0} TYPE string DEFAULT '';
DEFINE FIELD IF NOT EXISTS gender ON TABLE {0} TYPE option<string>;
DEFINE FIELD IF NOT EXISTS role ON TABLE {0} TYPE string DEFAULT 'User';
DEFINE FIELD IF NOT EXISTS enabled ON TABLE {0} TYPE bool DEFAULT true;
DEFINE INDEX name_index ON TABLE {0} COLUMNS name UNIQUE;
DEFINE INDEX email_index ON TABLE {0} COLUMNS email UNIQUE;

DEFINE TABLE IF NOT EXISTS {1} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE {1} TYPE string;
DEFINE FIELD IF NOT EXISTS exp ON TABLE {1} TYPE string;

DEFINE ACCESS IF NOT EXISTS {0} ON DATABASE TYPE RECORD
    SIGNUP ( CREATE {0} SET name = $name, email = $email, password = crypto::argon2::generate($password) )
    SIGNIN ( SELECT * FROM {0} WHERE name = $name AND crypto::argon2::compare(password, $password) )
    AUTHENTICATE {{
        INSERT INTO {1} {{ id: $token.jti, user_id : <string>meta::id($auth.id), exp: <string>$token.exp}};
        RETURN $auth;
    }}
    DURATION FOR TOKEN 15m, FOR SESSION 12h
;
",
            USER_TABLE, TOKEN_TABLE
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
            access: USER_TABLE,
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
            access: USER_TABLE,
            params: data.0.clone(),
        })
        .await?
        .into_insecure_token();

    let _ = set_db().await?;

    Ok(Json(Token::new(jwt)))
}

pub async fn authenticate(header: &HeaderMap) -> Result<Json<Token>, ServerError> {
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

    let jwt: String = get_jti_raw(&jwt).unwrap();

    Ok(Json(Token::new(jwt)))
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

pub async fn get_user_from_header(header: &HeaderMap) -> Result<Json<Option<User>>, ServerError> {
    let jwt: String = authenticate(header).await?.0.token;

    let token: Option<TokenData> = CONN.select((TOKEN_TABLE, jwt)).await?;

    if token.is_none() {
        return Err(ServerError::Unknown(String::from(
            "AUTH ERROR : User does not exist.",
        )));
    }

    Ok(Json(
        CONN.select((USER_TABLE, token.unwrap().user_id.trim()))
            .await?,
    ))
}

pub async fn get_user_from_id(id: Path<String>) -> Result<Json<Option<User>>, ServerError> {
    Ok(Json(CONN.select((USER_TABLE, id.trim())).await?))
}

pub async fn get_user(Extension(user): Extension<User>) -> Result<Json<User>, ServerError> {
    Ok(Json(user))
}

pub async fn list_all_users() -> Result<Json<Vec<User>>, ServerError> {
    Ok(Json(CONN.select(USER_TABLE).await?))
}

pub async fn list_all_tokens() -> Result<Json<Vec<TokenData>>, ServerError> {
    Ok(Json(CONN.select(TOKEN_TABLE).await?))
}

fn get_jti_raw(token: &str) -> Option<String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    let payload_b64 = parts[1];
    let decoded = BASE64_URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;

    json.get("jti")?.as_str().map(|s| s.to_string())
}
