// @generated automatically by Diesel CLI.

diesel::table! {
    events (id) {
        id -> Text,
        summary -> Text,
        description -> Nullable<Text>,
        status -> Nullable<Text>,
        start_datetime -> Text,
        end_datetime -> Text,
    }
}

diesel::table! {
    notifications (event_id) {
        event_id -> Text,
        enabled -> Bool,
        notification_time_from_start -> Integer,
    }
}

diesel::joinable!(notifications -> events (event_id));

diesel::allow_tables_to_appear_in_same_query!(
    events,
    notifications,
);
