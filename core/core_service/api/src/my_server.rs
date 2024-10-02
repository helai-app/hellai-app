use std::sync::Arc;

use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct MyServer {
    pub connection: Arc<DatabaseConnection>,
}
