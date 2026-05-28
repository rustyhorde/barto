-- Add up migration script here
CREATE TABLE IF NOT EXISTS output
(
    id          BIGINT UNSIGNED PRIMARY KEY NOT NULL AUTO_INCREMENT,
    timestamp   TIMESTAMP  NOT NULL,
    bartoc_uuid UUID       NOT NULL,
    bartoc_name TEXT       NOT NULL,
    cmd_uuid    UUID       NOT NULL,
    kind        VARCHAR(6) NOT NULL,
    data        TEXT       NOT NULL
);