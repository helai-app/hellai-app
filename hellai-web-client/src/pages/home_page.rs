#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::utilities::cookie_manager::WebClientCookieManager;

#[component]
pub fn HomeElement() -> Element {
    if WebClientCookieManager::get_cookie("my_key").is_none() {
        let navigator = use_navigator();
        navigator.replace("/login");
    }

    rsx! {"Home Page"}
}
