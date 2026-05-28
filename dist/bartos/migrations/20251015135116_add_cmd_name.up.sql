ALTER TABLE output_test ADD cmd_name VARCHAR(256) NOT NULL DEFAULT "unset" AFTER cmd_uuid;
ALTER TABLE output ADD cmd_name VARCHAR(256) NOT NULL DEFAULT "unset" AFTER cmd_uuid;