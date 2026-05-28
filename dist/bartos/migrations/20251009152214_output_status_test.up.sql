CREATE TABLE IF NOT EXISTS output_test
(
    id          BIGINT UNSIGNED PRIMARY KEY NOT NULL AUTO_INCREMENT,
    timestamp   TIMESTAMP  NOT NULL,
    bartoc_uuid UUID       NOT NULL,
    bartoc_name TEXT       NOT NULL,
    cmd_uuid    UUID       NOT NULL,
    kind        VARCHAR(6) NOT NULL,
    data        TEXT       NOT NULL
);

CREATE TABLE IF NOT EXISTS exit_status_test
(
    id          BIGINT  UNSIGNED PRIMARY KEY NOT NULL AUTO_INCREMENT,
    cmd_uuid    UUID                         NOT NULL,
    exit_code   TINYINT UNSIGNED             NOT NULL,
    success     BOOLEAN                      NOT NULL
);