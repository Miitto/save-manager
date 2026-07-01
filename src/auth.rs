use crate::{
    Route, USER,
    components::{Button, Checkbox, Label, LabeledInput},
};
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;

#[component]
pub fn AuthLayout() -> Element {
    let route = use_route::<crate::Route>();

    let is_login = matches!(route, crate::Route::Login {});

    let navigator = use_navigator();

    let mut update_user = use_action(move || async move {
        if let Ok(usr) = api::get_user().await {
            (*USER.write()) = usr;
        } else {
            (*USER.write()) = None;
        }

        Ok(()) as Result<(), ServerFnError>
    });

    use_effect(move || {
        update_user.call();
    });

    use_effect(move || {
        if update_user.value().is_some() && USER().is_some() {
            warn!("User is logged in, redirecting to home page");
            navigator.replace(Route::Saves {});
        }
    });

    rsx! {
        div { class: "flex flex-col items-center mt-10",
            div { class: "flex flex-row container justify-center items-center text-2xl border border-border rounded w-fit",
                Link { to: Route::Login {},
                    div {
                        class: if is_login { "bg-secondary-2 text-primary" },
                        class: "flex items-center justify-center cursor-pointer hover:underline w-60 h-20 rounded-l",
                        span { "Login" }
                    }
                }
                Link { to: Route::Register {},
                    div {
                        class: if !is_login { "bg-secondary-2 text-primary" },
                        class: "flex items-center justify-center cursor-pointer hover:underline w-60 h-20 rounded-r",
                        span { "Register" }
                    }
                }
            }
            Outlet::<Route> {}
        }
    }
}

#[component]
pub fn Login() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut remember = use_signal(|| false);
    let checkbox_state = use_memo(move || {
        Some(if remember() {
            CheckboxState::Checked
        } else {
            CheckboxState::Unchecked
        })
    });

    let mut login_user = use_action(move || async move {
        let usr = match api::login(
            username(),
            password(),
            if cfg!(feature = "desktop") {
                Some(true)
            } else {
                Some(remember())
            },
        )
        .await
        {
            Ok(usr) => usr,
            Err(e) => match e {
                ServerFnError::ServerError { message, .. } => {
                    return Ok(Some(message));
                }
                _ => {
                    return Ok(Some("An unknown error occurred".to_string()));
                }
            },
        };
        *USER.write() = Some(usr);
        Ok::<Option<String>, ServerFnError>(None)
    });

    let failure_message = login_user.value().and_then(|res| {
        res.ok().and_then(|s| s()).map(|msg| {
            rsx! {
                p { class: "col-span-full text-red-500 mt-2", {msg} }
            }
        })
    });

    rsx! {
        document::Title { "Login" }

        form {
            class: "flex flex-col gap-4 items-center p-4 container w-120 border border-border rounded mt-8",
            onsubmit: move |e| {
                e.prevent_default();
                login_user
                    .call()
            },
            LabeledInput {
                div_class: "w-full",
                id: "username",
                name: "username",
                required: true,
                placeholder: "Username",
                oninput: move |e: FormEvent| username.set(e.value()),
                "Username"
            }
            LabeledInput {
                div_class: "w-full",
                id: "password",
                name: "password",
                required: true,
                placeholder: "Password",
                r#type: "password",
                oninput: move |e: FormEvent| password.set(e.value()),
                "Password"
            }

            if cfg!(not(feature = "desktop")) {
                div { class: "flex flex-row gap-2 items-center",
                    Label { html_for: "remember", "Remember me" }
                    Checkbox {
                        id: "remember",
                        name: "remember",
                        checked: checkbox_state,
                        on_checked_change: move |state| {
                            remember.set(state == CheckboxState::Checked);
                        },
                    }
                }
            }

            {failure_message}
            Button { class: "w-full", "Login" }
        }
    }
}

#[component]
pub fn Register() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut remember = use_signal(|| false);
    let checkbox_state = use_memo(move || {
        Some(if remember() {
            CheckboxState::Checked
        } else {
            CheckboxState::Unchecked
        })
    });

    let mut register = use_action(move || async move {
        if password() != confirm_password() {
            return Ok(Some("Passwords do not match".to_string()));
        }
        let usr = match api::register(
            username(),
            password(),
            if cfg!(feature = "desktop") {
                Some(true)
            } else {
                Some(remember())
            },
        )
        .await
        {
            Ok(usr) => usr,
            Err(e) => match e {
                ServerFnError::ServerError { message, .. } => {
                    return Ok(Some(message));
                }
                _ => {
                    return Ok(Some("An unknown error occurred".to_string()));
                }
            },
        };
        *USER.write() = Some(usr);
        Ok::<Option<String>, ServerFnError>(None)
    });

    let failure_message = register.value().and_then(|res| {
        res.ok().and_then(|s| s()).map(|msg| {
            rsx! {
                p { class: "col-span-full text-red-500 mt-2", {msg} }
            }
        })
    });

    rsx! {
        document::Title { "Register" }

        form {
            class: "flex flex-col gap-4 items-center p-4 container w-120 border border-border rounded mt-8",
            onsubmit: move |e| {
                e.prevent_default();
                register.call();
            },
            LabeledInput {
                div_class: "w-full",
                id: "username",
                name: "username",
                required: true,
                placeholder: "Username",
                oninput: move |e: FormEvent| username.set(e.value()),
                "Username"
            }
            LabeledInput {
                div_class: "w-full",
                id: "password",
                name: "password",
                required: true,
                placeholder: "Password",
                r#type: "password",
                oninput: move |e: FormEvent| password.set(e.value()),
                "Password"
            }
            LabeledInput {
                div_class: "w-full",
                id: "confirm_password",
                name: "confirm_password",
                required: true,
                placeholder: "Confirm Password",
                r#type: "password",
                oninput: move |e: FormEvent| confirm_password.set(e.value()),
                "Confirm Password"
            }
            if cfg!(not(feature = "desktop")) {
                div { class: "flex flex-row gap-2 items-center",
                    Label { html_for: "remember", "Remember me" }
                    Checkbox {
                        id: "remember",
                        name: "remember",
                        checked: checkbox_state,
                        on_checked_change: move |state| {
                            remember.set(state == CheckboxState::Checked);
                        },
                    }
                }
            }
            {failure_message}
            Button { class: "w-full", "Register" }
        }
    }
}
