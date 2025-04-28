CREATE DATABASE IF NOT EXISTS messaging;
USE messaging;

CREATE TABLE IF NOT EXISTS users (
                       id INT AUTO_INCREMENT PRIMARY KEY,
                       username VARCHAR(255) NOT NULL UNIQUE,
                       password_hash VARCHAR(255) NOT NULL
);

CREATE TABLE IF NOT EXISTS friends (
                         user_id INT NOT NULL,
                         friend_id INT NOT NULL,
                         PRIMARY KEY (user_id, friend_id),
                         FOREIGN KEY (user_id) REFERENCES users(id),
                         FOREIGN KEY (friend_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS blocked_users (
                               user_id INT NOT NULL,
                               blocked_id INT NOT NULL,
                               PRIMARY KEY (user_id, blocked_id),
                               FOREIGN KEY (user_id) REFERENCES users(id),
                               FOREIGN KEY (blocked_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS friend_requests (
                                 id INT AUTO_INCREMENT PRIMARY KEY,
                                 sender_id INT NOT NULL,
                                 receiver_id INT NOT NULL,
                                 status ENUM('pending', 'accepted', 'rejected') DEFAULT 'pending' NOT NULL,
                                 sent_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
                                 UNIQUE KEY unique_request (sender_id, receiver_id),
                                 FOREIGN KEY (sender_id) REFERENCES users(id),
                                 FOREIGN KEY (receiver_id) REFERENCES users(id)
);


CREATE TABLE IF NOT EXISTS chats (
                       id INT AUTO_INCREMENT PRIMARY KEY,
                       name VARCHAR(255),
                       is_group BOOLEAN DEFAULT FALSE NOT NULL,
                       created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS chat_members (
                              chat_id INT NOT NULL,
                              user_id INT NOT NULL,
                              PRIMARY KEY (chat_id, user_id),
                              FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
                              FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS messages (
                          id INT AUTO_INCREMENT PRIMARY KEY,
                          chat_id INT NOT NULL,
                          sender_id INT NOT NULL,
                          content TEXT NOT NULL,
                          `read` BOOLEAN DEFAULT FALSE NOT NULL,
                          timestamp DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
                          FOREIGN KEY (chat_id) REFERENCES chats(id),
                          FOREIGN KEY (sender_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS message_reads (
                               message_id INT NOT NULL,
                               user_id INT NOT NULL,
                               read_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
                               PRIMARY KEY (message_id, user_id),
                               FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
                               FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

