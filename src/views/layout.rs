use crate::db;

pub struct LayoutTemplate {
    pub logged_in_username: String,
    pub notes: Vec<db::Note>,
}
