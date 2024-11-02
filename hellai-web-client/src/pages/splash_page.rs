#![allow(non_snake_case)]

use std::time::Duration;

use dioxus::prelude::*;
// use dioxus_logger::tracing::info;
use tonic_web_wasm_client::Client;

use crate::{
    app_state::AppState,
    app_structs::{
        project_info::{ProjectInfo, UserProjectRole},
        user_data::UserData,
    },
    components::toast::{ToastFrame, ToastInfo, ToastManager},
    helai_api_core_service::{
        user_service_client::UserServiceClient, ClearUserResponse, EmptyGetUserRequest,
    },
    utilities::{constants::API_SERVER, cookie_manager::WebClientCookieManager},
    GLOBAL_APP_STATE,
};

#[component]
pub fn SplashElement() -> Element {
    let toast = use_context_provider(|| Signal::new(ToastManager::default()));

    rsx! {
        ToastFrame { manager: toast }
        SplashPopUpElement{},
        "Splash Page",
    }
}

#[component]
pub fn SplashPopUpElement() -> Element {
    let session_t = WebClientCookieManager::get_cookie("session_t"); // Retrieve session token from cookies
    let navigator = use_navigator(); // Navigator for redirecting based on session status
    let mut toast: Signal<ToastManager> = use_context(); // Get page toast manager

    // Check if session token exists
    match session_t {
        Some(_) => {
            // Fetch user data if session token is available
            let response = use_resource(move || try_get_user_data(session_t.clone().unwrap()));

            let read_response = response.read();

            // Read the asynchronous response for user data
            match &*read_response {
                // Handle successful user data response
                Some(response) => match response {
                    Ok(response) => {
                        // info!("Success to get use data.");

                        let _ = toast
                            .write()
                            .popup(ToastInfo::success(&"Succes auth".to_string(), "Success"));

                        // Clone and populate global state with user data
                        let user_data = response.clone();

                        *GLOBAL_APP_STATE.write() = AppState::Auth(UserData {
                            id: user_data.user_id,
                            email: user_data.email,
                            projects: user_data
                                .user_projects
                                .into_iter()
                                .map(|project| {
                                    let user_role = project.user_role.expect("Can't be None");
                                    ProjectInfo {
                                        id: project.project_id,
                                        name: project.project_name,
                                        role: UserProjectRole {
                                            id: user_role.role_id,
                                            name: user_role.name,
                                            description: user_role.description,
                                        },
                                    }
                                })
                                .collect(),
                        });

                        // Delay for show popUp
                        let _ = use_resource(move || async move {
                            async_std::task::sleep(Duration::from_millis(1000)).await;
                            navigator.replace("/");
                        });

                        rsx! {""}
                    }

                    // Handle case where response returns an error
                    Err(err_message) => {
                        // info!("Failed to get use data {}.", err_message);

                        let _ = toast.write().popup(ToastInfo::error(err_message, "Error"));

                        *GLOBAL_APP_STATE.write() = AppState::UnAuth;

                        // Delay for show popUp
                        let _ = use_resource(move || async move {
                            async_std::task::sleep(Duration::from_millis(1000)).await;
                            navigator.replace("/");
                        });
                        rsx! {""}
                    }
                },
                None => {
                    // info!("Awaiting...");
                    rsx! {""}
                }
            }
        }
        None => {
            // info!("Session token not found.");
            *GLOBAL_APP_STATE.write() = AppState::UnAuth;
            navigator.replace("/");
            rsx! {""}
        }
    }
}

async fn try_get_user_data(token: String) -> Result<ClearUserResponse, String> {
    let base_url = API_SERVER.to_string(); // URL of the gRPC-web server
    let mut query_client = UserServiceClient::new(Client::new(base_url));

    let mut request = tonic::Request::new(EmptyGetUserRequest {});

    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", token)
            .parse()
            .expect("Failed to parse authorization token"),
    );

    let response = query_client.get_user_data(request).await;

    match response {
        Ok(res) => {
            let res = res.into_inner();

            Ok(res)
        }
        Err(_) => Err("Failed to get response".to_string()),
    }
}
