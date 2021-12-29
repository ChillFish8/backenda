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
CREATE TABLE IF NOT EXISTS notifications (
    id uuid,
    recipient_id bigint,
    title text,
    description text,
    created_on timestamp,
    icon text,
    PRIMARY KEY ( id, recipient_id )
);
--
CREATE TABLE IF NOT EXISTS rooms (
    id uuid,
    guild_id bigint,
    owner_id bigint,
    active_playlist uuid,
    playing_now uuid,
    title text,
    topic text,
    is_public boolean,
    invite_only boolean,
    active boolean,
    banner text,
    created_on timestamp,
    PRIMARY KEY ( id, owner_id, active )
);