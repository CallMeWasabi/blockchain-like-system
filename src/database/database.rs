use std::sync::Arc;

use mongodb::{self, options::ClientOptions, Client, Database};

use crate::setting::Setting;

pub async fn db_connect(setting: Arc<Setting>) -> mongodb::error::Result<Database> {
    let mut client_options: ClientOptions = ClientOptions::parse(setting.get_db_url()).await?;

    client_options.app_name = Some("rust_chain".to_string());

    let client = Client::with_options(client_options)?;
    let db = client.database(&setting.database.dbname);

    return Ok(db);
}