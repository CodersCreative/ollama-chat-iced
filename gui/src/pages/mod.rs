use crate::pages::setup::SetupPage;

pub mod setup;

#[derive(Debug, Clone)]
pub enum Pages {
    Setup(SetupPage),
}

impl Default for Pages {
    fn default() -> Self {
        Self::Setup(SetupPage::default())
    }
}
