ALTER TABLE requests
	MODIFY COLUMN body Nullable(String),
	MODIFY COLUMN data Nullable(String);
