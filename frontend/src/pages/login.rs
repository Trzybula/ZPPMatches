use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::{LocalStorage, Storage};
use web_sys::HtmlInputElement;
use shared::LoginResponse;

fn api(path: &str) -> String {
    format!("/api{}", path)
}

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let email = use_state(|| "".to_string());
    let password = use_state(|| "".to_string());
    let message = use_state(|| "".to_string());

    let on_submit = {
        let email_state = email.clone();
        let password_state = password.clone();
        let message = message.clone();

    Callback::from(move |_| {
        let email = (*email_state).clone();
        let password = (*password_state).clone();
        let message = message.clone();

            spawn_local(async move {
                let body = serde_json::json!({
                    "email": email,
                    "password": password
                });

                let req = Request::post(&api("/login"))
                    .header("Content-Type", "application/json")
                    .body(body.to_string())
                    .expect("build request");

                match req.send().await {
                    Ok(resp) => match resp.json::<LoginResponse>().await {
                        Ok(parsed) => {
                            message.set(parsed.message.clone());

                            if parsed.ok {
                                if let Some(session) = parsed.session_id.clone() {
                                    let _ = LocalStorage::set("session_id", session.clone());

                                    let role = parsed.role.clone().unwrap_or_default().to_lowercase();
                                    if role == "company" {
                                        web_sys::window()
                                            .unwrap()
                                            .location()
                                            .set_href(&format!("/dashboard/company?session_id={}", session))
                                            .unwrap();
                                    } else if role == "group" {
                                        web_sys::window()
                                            .unwrap()
                                            .location()
                                            .set_href(&format!("/dashboard/group?session_id={}", session))
                                            .unwrap();
                                    } else if role == "admin" {
                                        web_sys::window()
                                            .unwrap()
                                            .location()
                                            .set_href(&format!("/admin?session_id={}", session))
                                            .unwrap();
                                    } else {
                                        message.set("Login ok but unknown role".into());
                                    }
                                }
                            }
                        }
                        Err(_) => message.set("Parse error".into()),
                    },
                    Err(err) => message.set(format!("Error: {:?}", err)),
                }
            });
        })
    };

    html! {
        <div>
            <h2>{ "Login" }</h2>

            <input
                placeholder="email"
                value={(*email).clone()}
                oninput={{
                    let email = email.clone();
                    Callback::from(move |e: InputEvent| {
                        let val = e.target_unchecked_into::<HtmlInputElement>().value();
                        email.set(val);
                    })
                }}
            />

            <input
                type="password"
                placeholder="password"
                value={(*password).clone()}
                oninput={{
                    let password = password.clone();
                    Callback::from(move |e: InputEvent| {
                        let val = e.target_unchecked_into::<HtmlInputElement>().value();
                        password.set(val);
                    })
                }}
            />

            <button onclick={on_submit}>{ "Log in" }</button>

            <p>{ (*message).clone() }</p>
        </div>
    }
}
