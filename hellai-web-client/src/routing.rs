use crate::pages::home_page::*;
use crate::pages::log_in_page::*;
use dioxus::prelude::*;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum SystemRoute {
    #[route("/")]
    HomeElement {},
    #[route("/login")]
    LogInElement {},
}
