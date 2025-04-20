use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, NotSet};
use crate::{entity, utils};
use utils::errors::server_error::ServerError;
use crate::entity::users::Model;
use crate::utils::json_models::auth_models::AuthResponseModel;

pub async fn get_user_by_username(username: String, db: Arc<DatabaseConnection>) -> Result<Option<Model>, ServerError> {
    entity::users::Entity::find().filter(entity::users::Column::Username.eq(username.clone())).one(&*db).await.map_err(|err| ServerError::DatabaseError(err))
}

pub async fn get_user_by_id(id: i32, db: Arc<DatabaseConnection>) -> Result<Option<Model>, ServerError> {
    entity::users::Entity::find().filter(entity::users::Column::Id.eq(id)).one(&*db).await.map_err(|err| ServerError::DatabaseError(err))
}