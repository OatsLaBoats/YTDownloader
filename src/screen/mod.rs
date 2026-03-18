pub mod home;
pub mod update;

pub enum Screen {
    Update(update::Screen),
    Home,
}
