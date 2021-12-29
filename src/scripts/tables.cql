CREATE TABLE IF NOT EXISTS users (
    id bigint,
    username text,
    avatar text,
    updated_on timestamp,
    access_servers map<bigint, boolean>,
    PRIMARY KEY ( id )
);
--
CREATE TABLE IF NOT EXISTS guilds (
    id bigint,
    name text,
    icon text,
    updated_on timestamp,
    PRIMARY KEY ( id )
);
--
CREATE TABLE IF NOT EXISTS access_tokens (
    user_id bigint,
    access_token text,
    PRIMARY KEY ( access_token )
)
WITH DEFAULT_TIME_TO_LIVE = 2419200;
--
