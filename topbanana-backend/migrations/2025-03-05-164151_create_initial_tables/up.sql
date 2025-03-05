
CREATE TABLE developers (
       id SERIAL PRIMARY KEY,
       name VARCHAR(100) NOT NULL,
       email VARCHAR(100) NOT NULL,
       url VARCHAR(100),
       is_admin BOOLEAN NOT NULL DEFAULT false,
       api_key VARCHAR(100) UNIQUE,
       UNIQUE NULLS NOT DISTINCT (name, email, url)
);

CREATE TABLE games (
       id serial PRIMARY KEY,
       developer_id INTEGER NOT NULL REFERENCES developers (id),
       game_uuid UUID NOT NULL UNIQUE,
       game_secret_key VARCHAR(100) NOT NULL,
       name VARCHAR(100) NOT NULL
);

CREATE TABLE highscore_tables (
       id serial PRIMARY KEY,
       game_id INTEGER NOT NULL REFERENCES games (id),
       name VARCHAR(100) NOT NULL,
       table_uuid UUID NOT NULL UNIQUE,
       maximum_scores_retained INTEGER DEFAULT NULL,
       UNIQUE (game_id, name)
);

CREATE TABLE highscore_table_entries (
       id serial PRIMARY KEY,
       highscore_table_id INTEGER NOT NULL REFERENCES highscore_tables (id),
       player_name VARCHAR(100) NOT NULL,
       player_score FLOAT NOT NULL,
       player_score_metadata TEXT,
       creation_timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX highscore_index_by_value ON highscore_table_entries (highscore_table_id, player_score DESC);
