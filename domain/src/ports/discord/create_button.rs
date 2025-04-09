use tracing::instrument;

#[derive(Debug)]
pub struct CreateButton {
    pub label: String,
    pub data: ButtonKind,
}

impl CreateButton {
    #[instrument(level = "trace", skip(label, button_id))]
    pub fn new(label: impl Into<String>, button_id: impl Into<String>) -> Self {
        CreateButton {
            label: label.into(),
            data: ButtonKind::NonLink {
                button_id: ButtonId(button_id.into()),
            },
        }
    }

    #[instrument(level = "trace", skip(label, url))]
    pub fn new_link(label: impl Into<String>, url: impl Into<String>) -> Self {
        CreateButton {
            label: label.into(),
            data: ButtonKind::Link { url: url.into() },
        }
    }
}

#[derive(Debug)]
pub enum ButtonKind {
    NonLink { button_id: ButtonId },
    Link { url: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct ButtonId(pub String);
