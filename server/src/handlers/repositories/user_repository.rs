use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, NotSet, Condition};
use sea_orm::sea_query::Expr;
use crate::{entity, utils};
use utils::errors::server_error::ServerError;
use crate::entity::sea_orm_active_enums::Status;
use crate::entity::users;

pub async fn register_user(username: String, hashed: String, db: Arc<DatabaseConnection>) -> Result<entity::users::ActiveModel, ServerError> {
    // Create a new user
    let new_user = entity::users::ActiveModel { 
        id: NotSet,
        username: Set(username),
        password_hash: Set(hashed.clone()),
    };

    // Save the user to DB
    new_user.save(&*db).await.map_err(|err| ServerError::DatabaseError(err))
}

pub async fn get_user_by_username(username: String, db: Arc<DatabaseConnection>) -> Result<Option<entity::users::Model>, ServerError> {
    entity::users::Entity::find().filter(entity::users::Column::Username.eq(username.clone())).one(&*db).await.map_err(|err| ServerError::DatabaseError(err))
}

pub async fn get_user_by_id(id: i32, db: Arc<DatabaseConnection>) -> Result<Option<entity::users::Model>, ServerError> {
    entity::users::Entity::find().filter(entity::users::Column::Id.eq(id)).one(&*db).await.map_err(|err| ServerError::DatabaseError(err))
}

pub async fn send_friend_request(
    sender_id: i32,
    receiver_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<(), ServerError> {

    // Check if these users are already friends
    if let Some(_) = get_friendship(sender_id, receiver_id, db.clone()).await? {
        return Err(ServerError::AlreadyFriends)
    }

    // Check if this exact request already exists
    if let Some(_) = get_friend_request(sender_id, receiver_id, db.clone()).await? {
        return Ok(())
    }

    // Check for a reverse request
    let reverse = get_friend_request(receiver_id, sender_id, db.clone()).await?;

    if let Some(existing_request) = reverse {
        // Accept the reverse request
        let mut model: entity::friend_requests::ActiveModel = existing_request.into();
        model.status = Set(Status::Accepted);
        model.update(&*db).await?;

        // Create mutual friendships
        for (u1, u2) in [(sender_id, receiver_id), (receiver_id, sender_id)] {
            create_friendship(u1, u2, db.clone()).await?;
        }

        return Ok(());
    }

    // No existing or reverse request, create a new one
    let new_request = entity::friend_requests::ActiveModel {
        sender_id: Set(sender_id),
        receiver_id: Set(receiver_id),
        status: Set(Status::Pending),
        ..Default::default()
    };

    new_request.insert(&*db).await.map_err(ServerError::DatabaseError)?;

    Ok(())
}

pub async fn get_friend_request(sender_id: i32, receiver_id: i32, db: Arc<DatabaseConnection>) -> Result<Option<entity::friend_requests::Model>, ServerError> {
    Ok(entity::friend_requests::Entity::find()
        .filter(entity::friend_requests::Column::SenderId.eq(sender_id))
        .filter(entity::friend_requests::Column::ReceiverId.eq(receiver_id))
        .filter(entity::friend_requests::Column::Status.eq(Status::Pending))
        .one(&*db)
        .await?)
}


pub async fn update_friend_request_status(sender_id: i32, receiver_id: i32, status: Status, db: Arc<DatabaseConnection>) -> Result<(), ServerError> {
    if status == Status::Rejected {
        entity::friend_requests::Entity::delete_many()
            .filter(entity::friend_requests::Column::SenderId.eq(sender_id))
            .filter(entity::friend_requests::Column::ReceiverId.eq(receiver_id))
            .exec(&*db)
            .await?;
    } else {
        entity::friend_requests::Entity::update_many()
            .col_expr(entity::friend_requests::Column::Status, Expr::value(status))
            .filter(entity::friend_requests::Column::SenderId.eq(sender_id))
            .filter(entity::friend_requests::Column::ReceiverId.eq(receiver_id))
            .exec(&*db)
            .await?;
    }

    Ok(())
}

pub async fn create_friendship(u1: i32, u2: i32, db: Arc<DatabaseConnection>) -> Result<(), ServerError> {
    let friendship = entity::friends::ActiveModel {
        user_id: Set(u1),
        friend_id: Set(u2),
    };
    friendship.insert(&*db).await?;
    Ok(())
}

pub async fn get_user_blocked(sender_id: i32, receiver_id: i32, db: Arc<DatabaseConnection>) -> Result<Option<entity::blocked_users::Model>, ServerError> {
    entity::blocked_users::Entity::find()
        .filter(
            Condition::any()
                .add(entity::blocked_users::Column::UserId.eq(sender_id).and(entity::blocked_users::Column::BlockedId.eq(receiver_id)))
                .add(entity::blocked_users::Column::UserId.eq(receiver_id).and(entity::blocked_users::Column::BlockedId.eq(sender_id)))
        )
        .one(&*db)
        .await.map_err(|err| ServerError::DatabaseError(err))
}

pub async fn block_user(user_id: i32, blocked_id: i32, db: Arc<DatabaseConnection>) -> Result<(), ServerError> {
    let new_block = entity::blocked_users::ActiveModel {
        user_id: Set(user_id),
        blocked_id: Set(blocked_id),
    };

    new_block.insert(&*db).await.map_err(ServerError::DatabaseError)?;

    Ok(())
}

pub async fn delete_friendship(u1: i32, u2: i32, db: Arc<DatabaseConnection>) -> Result<(), ServerError> {
    entity::friends::Entity::delete_many()
        .filter(
            Condition::any()
                .add(entity::friends::Column::UserId.eq(u1).and(entity::friends::Column::FriendId.eq(u2)))
                .add(entity::friends::Column::UserId.eq(u2).and(entity::friends::Column::FriendId.eq(u1)))
        )
        .exec(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;

    Ok(())
}

pub async fn get_friendship(u1: i32, u2: i32, db: Arc<DatabaseConnection>) -> Result<Option<entity::friends::Model>, ServerError> {
    entity::friends::Entity::find()
        .filter(
            Condition::any()
                .add(entity::friends::Column::UserId.eq(u1).and(entity::friends::Column::FriendId.eq(u2)))
                .add(entity::friends::Column::UserId.eq(u2).and(entity::friends::Column::FriendId.eq(u1)))
        )
        .one(&*db).await.map_err(|err| ServerError::DatabaseError(err))
}

pub async fn delete_friend_requests(u1: i32, u2: i32, db: Arc<DatabaseConnection>) -> Result<(), ServerError> {
    entity::friend_requests::Entity::delete_many()
        .filter(
            Condition::any()
                .add(entity::friend_requests::Column::SenderId.eq(u1).and(entity::friend_requests::Column::ReceiverId.eq(u2)))
                .add(entity::friend_requests::Column::SenderId.eq(u2).and(entity::friend_requests::Column::ReceiverId.eq(u1)))
        )
        .exec(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;

    Ok(())
}

pub async fn get_user_friends(user_id: i32, db: Arc<DatabaseConnection>) -> Result<Vec<entity::friends::Model>, ServerError> {
    let friends: Vec<entity::friends::Model> = entity::friends::Entity::find().filter(entity::friends::Column::UserId.eq(user_id)).all(&*db).await.map_err(|err| ServerError::DatabaseError(err))?;
    Ok(friends)
}

pub async fn get_users_from_list(ids: Vec<i32>, db: Arc<DatabaseConnection>) -> Result<Vec<entity::users::Model>, ServerError> {
    let users: Vec<entity::users::Model> = entity::users::Entity::find().filter(entity::users::Column::Id.is_in(ids)).all(&*db).await.map_err(|err| ServerError::DatabaseError(err))?;
    Ok(users)
}

pub async fn get_user_friend_requests(user_id: i32, direction: Option<entity::friend_requests::Column>, db: Arc<DatabaseConnection>) -> Result<Vec<entity::friend_requests::Model>, ServerError> {
    if let Some(direction) = direction {
        let requests: Vec<entity::friend_requests::Model> = entity::friend_requests::Entity::find()
            .filter(direction.eq(user_id))
            .filter(entity::friend_requests::Column::Status.eq(Status::Pending))
            .all(&*db)
            .await?;
        return Ok(requests);
    }

    let incoming: Vec<entity::friend_requests::Model> = entity::friend_requests::Entity::find()
        .filter(entity::friend_requests::Column::ReceiverId.eq(user_id))
        .filter(entity::friend_requests::Column::Status.eq(Status::Pending))
        .all(&*db)
        .await?;

    let outgoing: Vec<entity::friend_requests::Model> = entity::friend_requests::Entity::find()
        .filter(entity::friend_requests::Column::SenderId.eq(user_id))
        .filter(entity::friend_requests::Column::Status.eq(Status::Pending))
        .all(&*db)
        .await?;

    let requests = incoming.into_iter().chain(outgoing.into_iter()).collect();

    Ok(requests)
}

pub async fn update_password(
    username: String,
    hashed_password: String,
    db: Arc<DatabaseConnection>,
) -> Result<(), ServerError> {


    // Find the user
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(username.clone()))
        .one(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;

    let Some(user) = user else {
        return Err(ServerError::UserNotFound);
    };

    // Update password hash
    let mut active: users::ActiveModel = user.into();
    active.password_hash = Set(hashed_password);

    // Save to DB
    active
        .save(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;

    Ok(())
}