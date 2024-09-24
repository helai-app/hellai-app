use core_error::core_errors::CoreErrors;
use sea_orm::{ActiveModelTrait, DbConn, EntityTrait, Set, TryIntoModel};

use crate::entity::{prelude, users};

pub struct UserQuery {}

impl UserQuery {
    pub async fn get_user_by_id(
        db: &DbConn,
        user_id: i32,
    ) -> Result<Option<users::Model>, CoreErrors> {
        let user = prelude::Users::find_by_id(user_id).one(db).await?;

        Ok(user)
    }

    pub async fn create_new_user(
        db: &DbConn,
        user_name: String,
    ) -> Result<users::Model, CoreErrors> {
        let user_data = users::ActiveModel {
            username: Set(user_name),
            is_active: Set(true),

            ..Default::default()
        };

        let user = user_data.save(db).await?.try_into_model()?;

        Ok(user)
    }
}
