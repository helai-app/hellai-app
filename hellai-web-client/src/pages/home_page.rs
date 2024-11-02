#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::{app_state::AppState, GLOBAL_APP_STATE};

#[component]
pub fn HomeElement() -> Element {
    let user_data = GLOBAL_APP_STATE.read();
    let navigator = use_navigator();

    // Router Guard.
    // Open splash if app is not Initial.
    // Open login if there's no user data.
    match &*user_data {
        AppState::Initial => {
            navigator.replace("/splash");
            return rsx! {""}; // Return early after navigation
        }
        AppState::Auth(user) => {
            // No need to create a mutable variable, just borrow user
            rsx! { "Home Page", "{user}" }
        }
        AppState::UnAuth => {
            navigator.replace("/login");
            return rsx! {""}; // Return early after navigation
        }
    }
}
