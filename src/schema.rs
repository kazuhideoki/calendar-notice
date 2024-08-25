// @generated automatically by Diesel CLI.

diesel::table! {
    events (id) {
        id -> Text,
        summary -> Text,
        description -> Nullable<Text>,
        status -> Nullable<Text>,
        start_datetime -> Timestamp,
        end_datetime -> Timestamp,
    }
}

diesel::table! {
    notifications (event_id) {
        event_id -> Text,
        enabled -> Bool,
        notification_sec_from_start -> Integer,
    }
}

diesel::table! {
    oauth_tokens (id) {
        id -> Text,
        access_token -> Text,
        expires_in -> Nullable<Timestamp>,
        refresh_token -> Nullable<Text>,
        scope -> Nullable<Text>,
        token_type -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(notifications -> events (event_id));

diesel::allow_tables_to_appear_in_same_query!(
    events,
    notifications,
    oauth_tokens,
);
