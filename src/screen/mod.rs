pub mod home;
pub mod update;

pub enum Screen {
    Update(update::Screen),
    Home(home::Screen),
}

impl Default for Screen {
    fn default() -> Self {
        Self::Update(update::Screen::default())
    }
}
