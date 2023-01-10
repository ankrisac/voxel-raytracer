/// Namespaced labels of the form
/// `foo.bar.baz`
pub struct Label {
    data: String,
}
impl Label {
    #[must_use]
    pub fn sublabel<S: AsRef<str>>(&self, s: S) -> Label {
        Self {
            data: self.data.clone() + "." + s.as_ref(),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.data.as_str()
    }

    #[must_use]
    pub fn to_owned(&self) -> Self {
        Self { data: self.data.clone() }
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
