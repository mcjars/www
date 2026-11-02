ALTER TABLE requests
	UPDATE data = replaceAll(replaceAll(data, '"LEGACYFABRIC"', '"LEGACY_FABRIC"'), '"LOOHPLIMBO"', '"LOOHP_LIMBO"')
	WHERE data LIKE '%"LEGACYFABRIC"%' OR data LIKE '%"LOOHPLIMBO"%';
