CREATE TABLE IF NOT EXISTS user_authentication_requests(
    csrf_token VARCHAR(255) NOT NULL,
    user_id BIGINT NOT NULL,
    requested_at TIMESTAMP NOT NULL,
    PRIMARY KEY (csrf_token)
);
