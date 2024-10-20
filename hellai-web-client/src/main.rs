#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use routing::SystemRoute;

mod pages;
mod routing;

// #[derive(Clone, Routable, Debug, PartialEq)]
// enum Route {
//     #[route("/")]
//     Home {},
//     #[route("/blog/:id")]
//     Blog { id: i32 },
// }

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");

    info!("starting app");
    launch(App);
}

fn App() -> Element {
    rsx! {
        Router::<SystemRoute> {}
    }
}

// #[component]
// fn Home() -> Element {
//     let mut count = use_signal(|| 0);

//     // const ASSET: manganis::ImageAsset = manganis::mg!(image("./assets/images/auth_bg.png"));

//     rsx! {
//         div {
//             class: "container main-body",
//             div {
//                 class: "form-container",
//                 div {
//                     class: "form-header",
//                     h1 { "Welcome" }
//                     p { "Enter your login and password to sign in" }
//                 }

//                 form {
//                     div {
//                         class: "form-group",
//                         label { r#for: "login", "Login" }
//                         input {
//                             r#type: "email",
//                             id: "login",
//                             name: "login",
//                             placeholder: "Your login",
//                             required: true,
//                         }
//                     }

//                     div {
//                         class: "form-group",
//                         label { r#for: "password", "Password" }
//                         div {
//                             class: "input-wrapper",
//                             input {
//                                 r#type: "password",
//                                 id: "password",
//                                 name: "password",
//                                 placeholder: "Your password",
//                                 required: true,
//                             }
//                             span {
//                                 class: "eye-icon",
//                                 image {
//                                     id: "eyeIcon",
//                                     href: "./icons/icon_eye_open.svg",
//                                 }
//                             }
//                         }
//                     }

//                     div {
//                         class: "form-options",
//                         label {
//                             class: "switch",
//                             input {
//                                 r#type: "checkbox",
//                                 id: "remember",
//                                 name: "remember",
//                             }
//                             span {
//                                 class: "slider",
//                             }
//                         }
//                         label {
//                             class: "switch-label",
//                             r#for: "remember",
//                             "Remember me"
//                         }
//                     }

//                     button {
//                         class: "form-button",
//                         r#type: "submit",
//                         "SIGN IN"
//                     }
//                 }

//                 div {
//                     class: "signup",
//                     "Don't have an account? "
//                     a { href: "#", "Sign up" }
//                 }
//             }

//             div {
//                 class: "right-image",
//                 div {
//                     class: "logo",
//                     "HellAI"
//                 }
//             }
//         }

//         footer {
//             div {
//                 class: "container",
//                 div {
//                     class: "footer-content",
//                     p { "© 2025, Crafted with ❤️ for the HellAI project. Developed as an open-source initiative." }
//                 }

//                 div {
//                     class: "footer-navigation",
//                     a { href: "#", "GitHub" }
//                     a { href: "#", "Simmmple" }
//                     a { href: "#", "Blog" }
//                     a { href: "#", "Licence" }
//                 }
//             }
//         }
//     }
// }
