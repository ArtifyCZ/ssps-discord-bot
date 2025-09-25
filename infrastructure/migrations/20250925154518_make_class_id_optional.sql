ALTER TABLE authenticated_users ALTER COLUMN class_id DROP NOT NULL;
ALTER TABLE archived_authenticated_users ALTER COLUMN class_id DROP NOT NULL;
