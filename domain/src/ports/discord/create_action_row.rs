use crate::ports::discord::CreateButton;

#[derive(Debug)]
pub enum CreateActionRow {
    Buttons { components: Vec<CreateButton> },
}

impl CreateActionRow {
    pub fn buttons(components: Vec<CreateButton>) -> Self {
        CreateActionRow::Buttons { components }
    }
}
