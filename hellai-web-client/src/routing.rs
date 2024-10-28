use crate::pages::home_page::HomeElement;
use crate::pages::log_in_page::LogInElement;
use crate::pages::splash_page::SplashElement;
use dioxus::prelude::*;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum SystemRoute {
    #[route("/")]
    HomeElement {},
    #[route("/login")]
    LogInElement {},
    #[route("/splash")]
    SplashElement {},
}
