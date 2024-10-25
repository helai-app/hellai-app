#![allow(non_snake_case)]

use dioxus::prelude::*;

// use helai_api_core_service::user_service_client::UserServiceClient;

use crate::{
    components::toast::{ToastFrame, ToastInfo, ToastManager},
    helai_api_core_service::{
        user_service_client::UserServiceClient, AuthenticateWithPasswordRequest,
    },
    utilities::cookie_manager::WebClientCookieManager,
};
use tonic_web_wasm_client::Client;

// use hyper_util::rt::TokioExecutor;
// use tonic_web::GrpcWebClientLayer;

#[component]
pub fn LogInElement() -> Element {
    use_context_provider(|| Signal::new(LogInPageState::Unset));
    let toast = use_context_provider(|| Signal::new(ToastManager::default()));

    rsx! {
        ToastFrame { manager: toast }
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

                    PasswordInput{}

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

                    PrimaryButton {}
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
                    class: "footer-navigation",
                    a { href: "https://github.com/helai-app/hellai-app", target: "_blank", "GitHub" }
                    a { href: "#", "Simmmple" }
                    a { href: "#", "Blog" }
                    a { href: "#", "Licence" }
                }

                div {
                    class: "footer-content",
                    p { "© 2025, Crafted with ❤️ for the HellAI project. Developed as an open-source initiative." }
                }
            }
        }
    }
}

#[component]
pub fn PasswordInput() -> Element {
    let mut is_password_visible = use_signal(|| false);

    rsx! {
        div {
            class: "form-group",
            label { r#for: "password", "Password" }
            div {
                class: "input-wrapper",
                input {
                    r#type: if is_password_visible() { "text" } else { "password" },
                    id: "password",
                    name: "password",
                    placeholder: "Your password",
                    required: true,
                }
                span {
                    class: "eye-icon",
                    onclick: move |_event| is_password_visible.set(!is_password_visible()),
                    img {
                        id: "eyeIcon",
                        src: if is_password_visible() {
                            "./icons/icon_eye_close.svg"
                        } else {
                            "./icons/icon_eye_open.svg"
                        },
                    }
                }
            }
        }
    }
}

#[component]
pub fn PrimaryButton() -> Element {
    // let preview_state = LogInPageState::Unset;
    let mut preview_state = consume_context::<Signal<LogInPageState>>();
    let mut toast: Signal<ToastManager> = use_context();

    rsx! {
        button {
            class: "form-button",
            disabled: match preview_state() {
                LogInPageState::Loading => true,
                _ => false
            },
            onclick: move |_| async move {

                *preview_state.write() = LogInPageState::Loading;

                let auth_result = try_auth("admin".to_string(), "Admin123".to_string()).await;

                match auth_result {
                    Ok(_) =>  {
                        let navigator = use_navigator();
                        navigator.replace("/");
                        *preview_state.write() = LogInPageState::Success
                    },
                    Err(err_message) =>  {
                        let _ = toast.write().popup(ToastInfo::error(&err_message, "Error"));

                        *preview_state.write() = LogInPageState::Failed;
                    },
                }
            },
            r#type: "submit",
            match preview_state() {
                LogInPageState::Loading => rsx!(div { class: "spinner" }),
                _ => rsx!("SIGN IN")
            },
        }

        // Conditionally render the error toast if there's an error

    }
}

// State
#[derive(Clone, Debug)]
enum LogInPageState {
    Unset,
    Loading,
    Success,
    Failed,
}

async fn try_auth(login: String, password: String) -> Result<(), String> {
    let base_url = "http://0.0.0.0:50052".to_string(); // URL of the gRPC-web server
    let mut query_client = UserServiceClient::new(Client::new(base_url));
    let request = tonic::Request::new(AuthenticateWithPasswordRequest {
        login: login.into(),
        password: password.into(),
    });

    let response = query_client.authenticate_with_password(request).await; // Execute your queries the same way as you do with defaule transport layer

    match response {
        Ok(res) => {
            let res = res.into_inner();
            WebClientCookieManager::set_cookie("session_t", &res.session_token, true, false, None);
            WebClientCookieManager::set_cookie("refresh_t", &res.refresh_token, true, false, None);
            Ok(())
        }
        Err(_) => Err("Failed to get response".to_string()),
    }
}
