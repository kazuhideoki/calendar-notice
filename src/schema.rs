// @generated automatically by Diesel CLI.

diesel::table! {
    events (id) {
        id -> Nullable<Text>,
        summary -> Text,
        description -> Nullable<Text>,
        status -> Nullable<Text>,
        start_datetime -> Text,
        end_datetime -> Text,
    }
}
