#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::utilities::cookie_manager::WebClientCookieManager;

#[component]
pub fn HomeElement() -> Element {
    let session_t = WebClientCookieManager::get_cookie("session_t");
    let refresh_t = WebClientCookieManager::get_cookie("refresh_t");

    if session_t.is_none() && refresh_t.is_none() {
        let navigator = use_navigator();
        navigator.replace("/login");
    }

    rsx! {"Home Page"}
}
