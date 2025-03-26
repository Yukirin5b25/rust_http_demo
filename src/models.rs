use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::shortlink)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ShortLink {
    pub id: i64,
    pub hash: String,
    pub url: String,
    pub expire_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::shortlink)]
pub struct NewShortlink<'a> {
    pub hash: &'a str,
    pub url: &'a str,
    pub expire_at: &'a NaiveDateTime,
}
