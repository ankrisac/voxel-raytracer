/// Namespaced labels of the form
/// `foo.bar.baz`
pub(crate) struct Label {
    pub(crate) data: String,
}
impl Label {
    fn sublabel<S: AsRef<str>>(&self, s: S) -> Label {
        Self {
            data: self.data.clone() + "." + s.as_ref(),
        }
    }
}

impl From<&str> for Label {
    fn from(s: &str) -> Self {
        Self { data: s.to_owned() }
    }
}

impl From<String> for Label {
    fn from(s: String) -> Self {
        Self { data: s }
    }
}

impl From<&String> for Label {
    fn from(s: &String) -> Self {
        Self { data: s.to_owned() }
    }
}
