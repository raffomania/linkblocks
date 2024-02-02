use crate::db::lists::List;

pub struct LayoutTemplate {
    pub logged_in_username: String,
    pub lists: Vec<List>,
}
