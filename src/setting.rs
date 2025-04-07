use std::sync::Arc;

use config::Config;

#[derive(Debug, Clone)]
pub struct Server {
    pub port: i64,
}

#[derive(Debug, Clone)]
pub struct Database {
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password: String,
    pub dbname: String,
}

#[derive(Debug, Clone)]
pub struct Setting {
    pub server: Server,
    pub database: Database,
}

impl Setting {
    pub fn new() -> Result<Arc<Setting>, config::ConfigError> {
        let settings = Config::builder()
            .add_source(config::File::with_name("Settings"))
            .build()
            .unwrap();

        return Ok(Arc::new(Setting {
            server: Server {
                port: settings.get_int("server.port").unwrap(),
            },
            database: Database {
                host: settings.get_string("database.host").unwrap(),
                port: settings.get_int("database.port").unwrap(),
                username: settings.get_string("database.username").unwrap(),
                password: settings.get_string("database.password").unwrap(),
                dbname: settings.get_string("database.dbname").unwrap(),
            },
        }));
    }

    pub fn get_db_url(&self) -> String {
        return format!(
            "mongodb://{}:{}@{}:{}",
            self.database.username, self.database.password, self.database.host, self.database.port
        );
    }
}
