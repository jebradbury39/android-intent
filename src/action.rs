/// Action to invoke with an intent
pub enum Action {
    Send,
    Edit,
    Chooser,
    GetContent,
}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &str {
        match self {
            Self::Send => "ACTION_SEND",
            Self::Edit => "ACTION_EDIT",
            Self::Chooser => "ACTION_CHOOSER",
            Self::GetContent => "ACTION_GET_CONTENT",
        }
    }
}
