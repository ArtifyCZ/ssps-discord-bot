use crate::ports::discord::CreateButton;
use tracing::instrument;

#[derive(Debug)]
pub enum CreateActionRow {
    Buttons { components: Vec<CreateButton> },
}

impl CreateActionRow {
    #[instrument(level = "trace", skip(components))]
    pub fn buttons(components: Vec<CreateButton>) -> Self {
        CreateActionRow::Buttons { components }
    }
}
