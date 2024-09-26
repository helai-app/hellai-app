use sea_orm::DatabaseConnection;

#[derive(Default)]
pub struct MyServer {
    pub connection: DatabaseConnection,
}
