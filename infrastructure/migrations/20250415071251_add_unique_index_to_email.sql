CREATE UNIQUE INDEX idx_authenticated_users_email ON authenticated_users (email);
ALTER TABLE authenticated_users ADD CONSTRAINT authenticated_users_email UNIQUE (email);
