CREATE TABLE IF NOT EXISTS file_requests (
	`id` FixedString(12),
	`organization_id` Nullable(Int32),
	`origin` LowCardinality(Nullable(String)),
	`method` Enum8('GET' = 1, 'POST' = 2, 'PUT' = 3, 'DELETE' = 4, 'PATCH' = 5, 'OPTIONS' = 6, 'HEAD' = 7),
	`path` String,
	`root` LowCardinality(String),
	`kind` Enum8('index' = 1, 'file' = 2, 'checksums' = 3),
	`extension` LowCardinality(String),
	`size` Int64,
	`bytes_sent` Int64,
	`cache_hit` Bool,
	`time` Int32,
	`status` Int16,
	`ip` IPv6,
	`continent` Nullable(FixedString(2)),
	`country` Nullable(FixedString(2)),
	`user_agent` LowCardinality(String),
	`created` DateTime64(3),

	`_partition_date` Date MATERIALIZED toDate(created)
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(_partition_date)
ORDER BY (_partition_date, root, id)
SETTINGS index_granularity = 8192, allow_nullable_key = 1;
