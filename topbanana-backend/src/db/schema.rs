// @generated automatically by Diesel CLI.

diesel::table! {
    developers (id) {
        id -> Int4,
        developer_uuid -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 100]
        url -> Nullable<Varchar>,
        is_admin -> Bool,
        #[max_length = 100]
        api_key -> Nullable<Varchar>,
    }
}

diesel::table! {
    games (id) {
        id -> Int4,
        developer_id -> Int4,
        game_uuid -> Uuid,
        #[max_length = 100]
        game_secret_key -> Varchar,
        #[max_length = 100]
        name -> Varchar,
    }
}

diesel::table! {
    highscore_table_entries (id) {
        id -> Int4,
        highscore_table_id -> Int4,
        #[max_length = 100]
        player_name -> Varchar,
        player_score -> Float8,
        player_score_metadata -> Nullable<Text>,
        creation_timestamp -> Timestamptz,
    }
}

diesel::table! {
    highscore_tables (id) {
        id -> Int4,
        game_id -> Int4,
        #[max_length = 100]
        name -> Varchar,
        table_uuid -> Uuid,
        maximum_scores_retained -> Nullable<Int4>,
    }
}

diesel::table! {
    historical_requests (id) {
        id -> Int4,
        request_uuid -> Uuid,
        timestamp -> Timestamptz,
    }
}

diesel::joinable!(games -> developers (developer_id));
diesel::joinable!(highscore_table_entries -> highscore_tables (highscore_table_id));
diesel::joinable!(highscore_tables -> games (game_id));

diesel::allow_tables_to_appear_in_same_query!(
    developers,
    games,
    highscore_table_entries,
    highscore_tables,
    historical_requests,
);
