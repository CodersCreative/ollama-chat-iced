pub mod chats;
use std::sync::LazyLock;

use crate::utils::get_path_settings;
use surrealdb::{
    engine::local::{Db, RocksDb},
    Result, Surreal,
};

static CONN: LazyLock<Surreal<Db>> = LazyLock::new(Surreal::init);

pub async fn init_db() -> Result<()> {
    let _ = CONN
        .connect::<RocksDb>(&get_path_settings("ochat.db".to_string()))
        .await?;
    CONN.use_ns("test").use_db("test").await
}
