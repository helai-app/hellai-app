use core_error::core_errors::CoreErrors;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set, TryIntoModel};

use crate::entity::{passwords, prelude, users};

pub struct UserQuery {}

impl UserQuery {
    pub async fn get_user_by_id(
        db: &DbConn,
        user_id: i32,
    ) -> Result<Option<users::Model>, CoreErrors> {
        let user = prelude::Users::find_by_id(user_id).one(db).await?;

        Ok(user)
    }

    pub async fn get_user_by_login(
        db: &DbConn,
        user_login: String,
    ) -> Result<Option<(users::Model, passwords::Model)>, CoreErrors> {
        let user_data = prelude::Users::find()
            .filter(users::Column::Username.eq(user_login))
            .find_also_related(prelude::Passwords)
            .one(db)
            .await?;

        let user_with_password = match user_data {
            Some(user_data) => match user_data.1 {
                Some(password) => Some((user_data.0, password)),
                None => {
                    return Err(CoreErrors::DatabaseServiceError(
                        "user_data_broken".to_string(),
                    ))
                }
            },
            None => None,
        };

        Ok(user_with_password)
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
