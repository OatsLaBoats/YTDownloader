pub mod home;
pub mod update;

pub enum Screen {
    Update(update::Screen),
    Home,
}

impl Default for Screen {
    fn default() -> Self {
        Self::Update(update::Screen::default())
    }
}
