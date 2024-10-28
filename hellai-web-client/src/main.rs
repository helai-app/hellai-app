#![allow(non_snake_case)]

use app_state::AppState;
use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use routing::SystemRoute;

mod app_state;
mod app_structs;
mod components;
mod pages;
mod routing;
mod utilities;

/// For init proto generation
pub mod helai_api_core_service {
    tonic::include_proto!("helai_api_core_service");
}

static GLOBAL_APP_STATE: GlobalSignal<AppState> = Signal::global(|| AppState::Initial);

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
