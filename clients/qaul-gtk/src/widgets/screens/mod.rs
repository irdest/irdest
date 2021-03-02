use relm_derive::Msg;

mod login;
pub use login::Login;

pub struct Model {}

#[derive(Debug, Msg)]
pub enum Command {
    Login { id: String, password: String },
}
