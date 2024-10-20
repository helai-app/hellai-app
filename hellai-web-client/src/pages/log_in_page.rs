#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, trace};

use crate::utilities::cookie_manager::WebClientCookieManager;

#[component]
pub fn LogInElement() -> Element {
    // WebClientCookieManager::set_cookie("my_key", "my_value", true, false, None);

    let my_key_cookie = WebClientCookieManager::get_cookie("my_key");

    info!("my_key_cookie: {:?}", my_key_cookie);

    rsx! {
        div {
            class: "container main-body",
            div {
                class: "form-container",
                div {
                    class: "form-header",
                    h1 { "Welcome" }
                    p { "Enter your login and password to sign in" }
                }

                form {
                    div {
                        class: "form-group",
                        label { r#for: "login", "Login" }
                        input {
                            r#type: "email",
                            id: "login",
                            name: "login",
                            placeholder: "Your login",
                            required: true,
                        }
                    }

                    div {
                        class: "form-group",
                        label { r#for: "password", "Password" }
                        div {
                            class: "input-wrapper",
                            input {
                                r#type: "password",
                                id: "password",
                                name: "password",
                                placeholder: "Your password",
                                required: true,
                            }
                            span {
                                class: "eye-icon",
                                img {
                                    id: "eyeIcon",
                                    src: "./icons/icon_eye_open.svg",
                                }
                            }
                        }
                    }

                    div {
                        class: "form-options",
                        label {
                            class: "switch",
                            input {
                                r#type: "checkbox",
                                id: "remember",
                                name: "remember",
                            }
                            span {
                                class: "slider",
                            }
                        }
                        label {
                            class: "switch-label",
                            r#for: "remember",
                            "Remember me"
                        }
                    }

                    button {
                        class: "form-button",
                        r#type: "submit",
                        "SIGN IN"
                    }
                }

                div {
                    class: "signup",
                    "Don't have an account? "
                    a { href: "#", "Sign up" }
                }
            }

            div {
                class: "right-image",
                div {
                    class: "logo",
                    "HellAI"
                }
            }
        }

        footer {
            div {
                class: "container",
                div {
                    class: "footer-content",
                    p { "© 2025, Crafted with ❤️ for the HellAI project. Developed as an open-source initiative." }
                }

                div {
                    class: "footer-navigation",
                    a { href: "#", "GitHub" }
                    a { href: "#", "Simmmple" }
                    a { href: "#", "Blog" }
                    a { href: "#", "Licence" }
                }
            }
        }
    }
}
