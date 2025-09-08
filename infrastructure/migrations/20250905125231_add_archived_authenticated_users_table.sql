CREATE TABLE IF NOT EXISTS archived_authenticated_users
(
    user_id                 BIGINT                      NOT NULL,
    archived_at             TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    access_token            TEXT                        NOT NULL,
    refresh_token           TEXT                        NOT NULL,
    access_token_expires_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    name                    VARCHAR(255)                NOT NULL,
    email                   VARCHAR(255)                NOT NULL,
    class_id                VARCHAR(8)                  NOT NULL,
    authenticated_at        TIMESTAMP                   NOT NULL,
    PRIMARY KEY (user_id, archived_at)
);
