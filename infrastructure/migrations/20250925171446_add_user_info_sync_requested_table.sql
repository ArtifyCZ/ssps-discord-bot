CREATE TABLE IF NOT EXISTS user_info_sync_requested
(
    user_id      BIGINT                      NOT NULL,
    queued_at    TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    low_priority BOOLEAN                     NOT NULL DEFAULT FALSE,
    PRIMARY KEY (user_id)
);
