use server::utils::jwt::encode_jwt;

#[cfg(test)]
mod tests {
    use sea_orm::{Database, DbBackend, Schema, ConnectionTrait, DatabaseConnection, EntityTrait};
    use std::sync::Arc;
    use server::entity::{chats, chat_members, messages, users, message_reads};
    use server::utils::jwt;
    use server::handlers::services::{chat_service, auth_service};
    use server::utils::jwt::encode_jwt;

    async fn setup_in_memory_db() -> Arc<DatabaseConnection> {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(DbBackend::Sqlite);

        db.execute(db.get_database_backend().build(&schema.create_table_from_entity(users::Entity))).await.unwrap();
        db.execute(db.get_database_backend().build(&schema.create_table_from_entity(chats::Entity))).await.unwrap();
        db.execute(db.get_database_backend().build(&schema.create_table_from_entity(chat_members::Entity))).await.unwrap();
        db.execute(db.get_database_backend().build(&schema.create_table_from_entity(messages::Entity))).await.unwrap();
        db.execute(db.get_database_backend().build(&schema.create_table_from_entity(message_reads::Entity))).await.unwrap();

        let db = Arc::new(db);
        auth_service::register("Alice".to_owned(), "Password".to_string(), db.clone()).await.expect("Failed to register in DB setup");
        auth_service::register("Bob".to_owned(), "Password".to_string(), db.clone()).await.expect("Failed to register in DB setup");
        auth_service::register("Dylan".to_owned(), "Password".to_string(), db.clone()).await.expect("Failed to register in DB setup");

        db
    }

    fn mock_jwt(user_id: i32) -> String {
        // Replace with actual JWT if needed; for now we just encode user_id for testing
        jwt::encode_jwt(user_id).unwrap()
    }

    #[tokio::test]
    async fn test_create_chat_and_send_message_flow() {
        let db = setup_in_memory_db().await;

        let jwt = auth_service::login("Alice".to_owned(), "Password".into(), db.clone()).await.unwrap().token;

        let create_result = chat_service::create_chat(jwt.clone(), Some("Test Chat".to_string()), false, vec![2], db.clone()).await;
        assert!(create_result.is_ok());

        let chat = chats::Entity::find().one(&*db).await.unwrap().unwrap();

        let send_result = chat_service::send_message(jwt.clone(), chat.id, "Hello World!".to_string(), db.clone()).await;
        assert!(send_result.is_ok());

        let messages = chat_service::get_chat_messages(jwt.clone(), chat.id, 0, 10, db.clone()).await.unwrap();
        assert_eq!(messages.messages.len(), 1);
        assert_eq!(messages.messages[0].content, "Hello World!");
    }

    #[tokio::test]
    async fn test_mark_read_and_unread_count() {
        let db = setup_in_memory_db().await;

        let jwt = mock_jwt(1);

        let _ = chat_service::create_chat(jwt.clone(), Some("Test Chat 2".to_string()), false, vec![1], db.clone()).await;
        let chat = chats::Entity::find().one(&*db).await.unwrap().unwrap();

        // Insert 3 messages
        for _ in 0..3 {
            let _ = chat_service::send_message(jwt.clone(), chat.id, "msg".to_string(), db.clone()).await;
        }

        let unread_before = chat_service::get_unread_chat_message_count(1, chat.id, db.clone()).await.unwrap();
        assert_eq!(unread_before, 3);

        let _ = chat_service::mark_messages_read(jwt.clone(), chat.id, db.clone()).await.unwrap();

        let unread_after = chat_service::get_unread_chat_message_count(1, chat.id, db.clone()).await.unwrap();
        assert_eq!(unread_after, 0);
    }

    #[tokio::test]
    async fn test_group_chat_read_tracking() {
        let db = setup_in_memory_db().await;

        let jwt_alice = encode_jwt(1).unwrap();
        let jwt_bob = encode_jwt(2).unwrap();
        let jwt_dylan = encode_jwt(3).unwrap();

        // Alice creates a group chat
        chat_service::create_chat(jwt_alice.clone(), Some("Study Group".into()), true, vec![2, 3], db.clone()).await.unwrap();
        let chat = chats::Entity::find().one(&*db).await.unwrap().unwrap();

        // Alice sends one message
        chat_service::send_message(jwt_alice.clone(), chat.id, "Hey team!".into(), db.clone()).await.unwrap();
        let message = messages::Entity::find().one(&*db).await.unwrap().unwrap();

        // Check unread count for all members
        let unread_bob = chat_service::get_unread_chat_message_count(2, chat.id, db.clone()).await.unwrap();
        let unread_dylan = chat_service::get_unread_chat_message_count(3, chat.id, db.clone()).await.unwrap();
        assert_eq!(unread_bob, 1);
        assert_eq!(unread_dylan, 1);

        // Bob reads the message
        chat_service::mark_messages_read(jwt_bob.clone(), chat.id, db.clone()).await.unwrap();

        let unread_bob_after = chat_service::get_unread_chat_message_count(2, chat.id, db.clone()).await.unwrap();
        let unread_dylan_after = chat_service::get_unread_chat_message_count(3, chat.id, db.clone()).await.unwrap();

        assert_eq!(unread_bob_after, 0);
        assert_eq!(unread_dylan_after, 1);
    }
}


