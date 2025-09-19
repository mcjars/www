CREATE TABLE IF NOT EXISTS requests (
	`id` FixedString(12),
	`organization_id` Nullable(Int32),
	`origin` LowCardinality(Nullable(String)),
	`method` Enum8('GET' = 1, 'POST' = 2, 'PUT' = 3, 'DELETE' = 4, 'PATCH' = 5, 'OPTIONS' = 6, 'HEAD' = 7),
	`path` LowCardinality(String),
	`time` Int32,
	`status` Int16,
	`body` JSON,
	`data` JSON,
	`ip` IPv6,
	`continent` Nullable(FixedString(2)),
	`country` Nullable(FixedString(2)),
	`user_agent` LowCardinality(String),
	`created` DateTime64(3),

	`_partition_date` Date MATERIALIZED toDate(created)
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(_partition_date)
ORDER BY (organization_id, _partition_date, id)
SETTINGS index_granularity = 8192, allow_nullable_key = 1;
