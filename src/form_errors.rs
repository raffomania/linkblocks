pub struct FormErrors(pub garde::Report);

impl FormErrors {
    pub fn filter(&self, path_str: &str) -> Vec<String> {
        let path_to_find = garde::Path::new(path_str);
        self.0
            .iter()
            .filter(|(path, _error)| path == &path_to_find)
            .map(|(_path, error)| error.to_string())
            .collect()
    }
}

impl From<garde::Report> for FormErrors {
    fn from(report: garde::Report) -> Self {
        FormErrors(report)
    }
}

impl Default for FormErrors {
    fn default() -> Self {
        Self(garde::Report::new())
    }
}
