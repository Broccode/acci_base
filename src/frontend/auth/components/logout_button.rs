use crate::frontend::auth::AuthState;
use leptos::*;

#[component]
pub fn LogoutButton(#[prop(into)] class: String, #[prop(into)] text: String) -> impl IntoView {
    let auth = use_context::<AuthState>().expect("AuthState not found in context");

    let on_click = move |_| {
        let _ = auth.logout();
    };

    view! {
        <button
            class=class
            on:click=on_click
        >
            {text}
        </button>
    }
}
