// @generated automatically by Diesel CLI.

diesel::table! {
    shortlink (id) {
        id -> Int8,
        hash -> Varchar,
        url -> Varchar,
        expire_at -> Timestamp,
    }
}
