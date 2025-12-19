use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use axum::{Extension, Json, extract::Path, http::HeaderMap};
use ochat_types::user::{SigninData, SignupData, Token, User};
use serde::{Deserialize, Serialize};
use surrealdb::opt::auth::Record;

pub mod route;

use crate::{CONN, DATABASE, NAMESPACE, errors::ServerError, utils::get_count};
const USER_TABLE: &str = "user";

static TOKENS: LazyLock<RwLock<HashMap<String, Token>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub async fn define_users() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS name ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS email ON TABLE {0} TYPE string ASSERT string::is::email($value);
DEFINE FIELD IF NOT EXISTS pass ON TABLE {0} TYPE string VALUE crypto::argon2::generate($value);
DEFINE FIELD IF NOT EXISTS bio ON TABLE {0} TYPE string DEFAULT '';
DEFINE FIELD IF NOT EXISTS gender ON TABLE {0} TYPE option<string>;
DEFINE FIELD IF NOT EXISTS role ON TABLE {0} TYPE string DEFAULT 'User';
DEFINE INDEX name_index ON TABLE {0} COLUMNS name UNIQUE;

DEFINE ACCESS account ON DATABASE TYPE RECORD
    SIGNUP ( CREATE {0} SET name = $name, email = $email, pass = $pass )
    SIGNIN ( SELECT * FROM {0} WHERE name = $name AND crypto::argon2::compare(pass, $pass) )
;
",
            USER_TABLE
        ))
        .await?;
    Ok(())
}
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TokenData {
    pub user_id: String,
    pub token: String,
}

pub async fn signup(data: Json<SignupData>) -> Result<Json<Token>, ServerError> {
    let jwt = CONN
        .signup(Record {
            namespace: NAMESPACE,
            database: DATABASE,
            access: "account",
            params: data.0.clone(),
        })
        .await?
        .into_insecure_token();

    if let Some(x) = get_user_from_name(data.0.name.clone()).await? {
        TOKENS.write().unwrap().insert(
            x.id.key().to_string(),
            Token {
                token: jwt.trim().to_string(),
            },
        );
    } else {
        let _ = CONN
            .query(&format!(
                "
                CREATE {0} SET name = '{1}', email = '{2}', pass = '{3}';
            ",
                USER_TABLE,
                data.0.name.trim(),
                data.0.email.trim(),
                data.0.pass
            ))
            .await?;
        if let Some(x) = get_user_from_name(data.0.name.clone()).await? {
            TOKENS.write().unwrap().insert(
                x.id.key().to_string(),
                Token {
                    token: jwt.trim().to_string(),
                },
            );
        }
    }

    let _ = add_token(data.0.name, jwt.clone()).await;

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

pub async fn add_token(name: String, jwt: String) -> Result<(), ServerError> {
    let user = match get_user_from_name(name).await? {
        Some(x) => x,
        None => {
            return Err(ServerError::Unknown(
                "AUTH ERROR : User not added.".to_string(),
            ));
        }
    };

    let _ = TOKENS.write().unwrap().insert(
        user.id.key().to_string(),
        Token {
            token: jwt.trim().to_string(),
        },
    );

    Ok(())
}

pub async fn signin(data: Json<SigninData>) -> Result<Json<Token>, ServerError> {
    let jwt = CONN
        .signin(Record {
            namespace: NAMESPACE,
            database: DATABASE,
            access: "account",
            params: data.0.clone(),
        })
        .await?
        .into_insecure_token();

    let _ = add_token(data.0.name, jwt.clone()).await;

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

    let _ = CONN.authenticate(jwt.clone()).await?;
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

    let token = TOKENS
        .read()
        .as_ref()
        .unwrap()
        .iter()
        .find(|x| &x.1.token == jwt.trim())
        .map(|x| x.0.trim().to_string());

    if token.is_none() {
        return Err(ServerError::Unknown(String::from(
            "AUTH ERROR : User does not exist.",
        )));
    }

    Ok(Json(CONN.select((USER_TABLE, token.unwrap())).await?))
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
    Ok(Json(
        TOKENS
            .read()
            .unwrap()
            .clone()
            .into_iter()
            .map(|x| TokenData {
                user_id: x.0,
                token: x.1.token,
            })
            .collect(),
    ))
}
