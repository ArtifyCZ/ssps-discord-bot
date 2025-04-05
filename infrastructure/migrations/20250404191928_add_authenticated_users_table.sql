CREATE TABLE IF NOT EXISTS authenticated_users (
    user_id BIGINT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    class_id VARCHAR(8) NOT NULL,
    authenticated_at TIMESTAMP NOT NULL,
    PRIMARY KEY (user_id)
);
