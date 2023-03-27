
pub enum Category {
    Openable,
}

impl AsRef<str> for Category {
    fn as_ref(&self) -> &str {
        match self {
            Self::Openable => "CATEGORY_OPENABLE",
        }
    }
}