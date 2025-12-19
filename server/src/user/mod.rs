use axum::{Extension, Json, extract::Path, http::HeaderMap};
use ochat_types::user::{SigninData, SignupData, Token, User};
use serde::{Deserialize, Serialize};
use surrealdb::opt::auth::Record;

pub mod route;

use crate::{CONN, DATABASE, NAMESPACE, errors::ServerError, utils::get_count};
const USER_TABLE: &str = "user";
const TOKEN_TABLE: &str = "token";

pub async fn define_users() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS name ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS email ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS pass ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS bio ON TABLE {0} TYPE string DEFAULT '';
DEFINE FIELD IF NOT EXISTS gender ON TABLE {0} TYPE option<string>;
DEFINE FIELD IF NOT EXISTS role ON TABLE {0} TYPE string DEFAULT 'User';


DEFINE ACCESS account ON DATABASE TYPE RECORD
    SIGNUP ( CREATE {0} SET name = $name, email = $email, pass = crypto::argon2::generate($pass) )
    SIGNIN ( SELECT * FROM {0} WHERE name = $name AND crypto::argon2::compare(pass, $pass) )
;

DEFINE TABLE IF NOT EXISTS {1} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE {1} TYPE string;
DEFINE FIELD IF NOT EXISTS token ON TABLE {1} TYPE string;
",
            USER_TABLE, TOKEN_TABLE
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

pub async fn add_token(name: String, jwt: String) -> Result<Option<TokenData>, ServerError> {
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
        return Err(ServerError::Unknown(
            "AUTH ERROR : User not added.".to_string(),
        ));
    }
    let user = user.remove(0);

    Ok(CONN
        .create(TOKEN_TABLE)
        .content(TokenData {
            user_id: user.id.key().to_string(),
            token: jwt.clone(),
        })
        .await?)
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

    let _ = add_token(data.0.name, jwt.clone());

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

pub async fn get_user_from_header(header: &HeaderMap) -> Result<Json<Option<User>>, ServerError> {
    let jwt: String = authenticate(header).await?.0.token;

    let mut token: Vec<TokenData> = CONN
        .query(&format!(
            "
                SELECT * FROM {0} WHERE token = '{1}' LIMIT 1;
            ",
            TOKEN_TABLE,
            jwt.trim(),
        ))
        .await?
        .take(0)?;

    if token.is_empty() {
        return Err(ServerError::Unknown(String::from(
            "AUTH ERROR : User does not exist.",
        )));
    }

    let token = token.remove(0);

    Ok(Json(CONN.select((USER_TABLE, token.user_id.trim())).await?))
}

pub async fn get_user_from_id(id: Path<String>) -> Result<Json<Option<User>>, ServerError> {
    Ok(Json(CONN.select((USER_TABLE, id.trim())).await?))
}

pub async fn get_user(Extension(user): Extension<User>) -> Result<Json<User>, ServerError> {
    Ok(Json(user))
}
