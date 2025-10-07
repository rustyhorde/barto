-- Add up migration script here
CREATE TABLE IF NOT EXISTS exit_status
(
    id          BIGINT  UNSIGNED PRIMARY KEY NOT NULL AUTO_INCREMENT,
    cmd_uuid    UUID                         NOT NULL,
    exit_code   TINYINT UNSIGNED             NOT NULL,
    success     BOOLEAN                      NOT NULL
);