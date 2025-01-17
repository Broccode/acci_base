use crate::frontend::auth::AuthState;
use leptos::*;
use web_sys::UrlSearchParams;

#[component]
pub fn AuthCallback() -> impl IntoView {
    let auth = use_context::<AuthState>().expect("AuthState not found in context");

    let handle_callback = create_action(move |code: &String| {
        let code = code.to_string();
        async move {
            match auth.handle_callback(&code).await {
                Ok(_) => {
                    // Redirect to home page after successful login
                    let window = web_sys::window().unwrap();
                    let _ = window.location().set_href("/");
                },
                Err(e) => {
                    log::error!("Failed to handle auth callback: {}", e);
                    // Handle error (show error message, redirect to error page, etc.)
                },
            }
        }
    });

    create_effect(move |_| {
        let window = web_sys::window().unwrap();
        let search = window.location().search().unwrap();
        let params = UrlSearchParams::new_with_str(&search).unwrap();

        if let Some(code) = params.get("code") {
            handle_callback.dispatch(code);
        }
    });

    view! {
        <div class="auth-callback">
            "Processing authentication..."
        </div>
    }
}
