use crate::{
    Route, USER,
    components::{Button, LabeledInput},
};
use dioxus::prelude::*;

#[component]
pub fn AuthLayout() -> Element {
    let route = use_route::<crate::Route>();

    let is_login = matches!(route, crate::Route::Login {});

    let navigator = use_navigator();

    let mut update_user = use_action(move || async move {
        if let Ok(usr) = api::get_user().await {
            (*USER.write()) = Some(usr);
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
                        class: if is_login { "bg-neutral-600 " },
                        class: "flex items-center justify-center cursor-pointer hover:underline w-60 h-20 rounded-l",
                        span { "Login" }
                    }
                }
                Link { to: Route::Register {},
                    div {
                        class: if !is_login { "bg-neutral-600 " },
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

    let mut login_user = use_action(move || async move {
        let usr = match api::login(username(), password()).await {
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
            class: "flex flex-col gap-4 items-center p-4 container w-120 border border-neutral-500/50 rounded mt-8",
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

    let mut register = use_action(move || async move {
        if password() != confirm_password() {
            return Ok(Some("Passwords do not match".to_string()));
        }
        let usr = match api::register(username(), password()).await {
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
            class: "flex flex-col gap-4 items-center p-4 container w-120 border border-neutral-500/50 rounded mt-8",
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
            {failure_message}
            Button { class: "w-full", "Register" }
        }
    }
}
