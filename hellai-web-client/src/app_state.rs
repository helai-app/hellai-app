use crate::app_structs::user_data::UserData;

pub enum AppState {
    Initial,
    Auth(UserData),
    UnAuth,
}
