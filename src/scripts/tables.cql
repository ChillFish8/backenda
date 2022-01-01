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
    PRIMARY KEY ( id, recipient_id, created_on )
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
    PRIMARY KEY ( id, owner_id, active )
);
--
CREATE TABLE IF NOT EXISTS playlists (
    id uuid,
    owner_id bigint,
    title text,
    description text,
    items set<uuid>,
    nsfw boolean,
    is_public boolean,
    banner text,
    votes int,
    PRIMARY KEY ( id, owner_id )
);
--
CREATE TABLE IF NOT EXISTS playlist_votes (
    user_id bigint,
    playlist_id uuid,
    PRIMARY KEY ( (user_id, playlist_id) )
)
WITH DEFAULT_TIME_TO_LIVE = 43200;
--
CREATE TABLE IF NOT EXISTS playlist_entries (
    id uuid,
    owner_id bigint,
    title text,
    description text,
    ref_link text,
    nsfw boolean,
    is_public boolean,
    votes int,
    PRIMARY KEY ( id, owner_id )
);
--
CREATE TABLE IF NOT EXISTS playlist_entries_votes (
    user_id bigint,
    entry_id uuid,
    PRIMARY KEY ( (user_id, entry_id) )
)
WITH DEFAULT_TIME_TO_LIVE = 43200;
--
CREATE TABLE IF NOT EXISTS user_vote_credits (
    user_id bigint,
    credits int,
    PRIMARY KEY ( user_id )
);

