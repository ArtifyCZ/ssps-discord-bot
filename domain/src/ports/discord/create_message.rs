use crate::ports::discord::CreateAttachment;

#[derive(Default, Debug)]
pub struct CreateMessage {
    pub content: Option<String>,
    pub attachments: Vec<CreateAttachment>,
}

impl CreateMessage {
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn add_file(mut self, attachment: CreateAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }
}
