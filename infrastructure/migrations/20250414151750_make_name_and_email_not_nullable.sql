ALTER TABLE authenticated_users ALTER COLUMN name SET NOT NULL;
ALTER TABLE authenticated_users ALTER COLUMN name DROP DEFAULT;
ALTER TABLE authenticated_users ALTER COLUMN email SET NOT NULL;
ALTER TABLE authenticated_users ALTER COLUMN email DROP DEFAULT;
