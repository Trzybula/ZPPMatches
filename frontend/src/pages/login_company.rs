use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::{LocalStorage, Storage};
use serde::Deserialize;
use web_sys::HtmlInputElement;

#[derive(Deserialize, Clone, Debug)]
struct LoginResponse {
    ok: bool,
    message: String,
    session_id: Option<String>,
}

#[function_component(LoginCompanyPage)]
pub fn login_company_page() -> Html {
    let email = use_state(|| "".to_string());
    let password = use_state(|| "".to_string());
    let message = use_state(|| "".to_string());

    let on_submit = {
        let email = email.clone();
        let password = password.clone();
        let message = message.clone();

        Callback::from(move |_| {
            let email = (*email).clone();
            let password = (*password).clone();
            let message = message.clone();

            spawn_local(async move {
                let body = serde_json::json!({
                    "email": email,
                    "password": password
                });

                let req = Request::post("http://localhost:3000/login/company")
                    .header("Content-Type", "application/json")
                    .body(body.to_string())
                    .expect("build request");

                match req.send().await {
                    Ok(resp) => match resp.json::<LoginResponse>().await {
                        Ok(parsed) => {
                            message.set(parsed.message.clone());

                            if parsed.ok {
                                if let Some(session) = parsed.session_id {
                                    let _ = LocalStorage::set("session_id", session.clone());

                                    web_sys::window()
                                        .unwrap()
                                        .location()
                                        .set_href(&format!("/dashboard/company?session_id={}", session))
                                        .unwrap();
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
            <h2>{ "Login (Company)" }</h2>

            <input
                placeholder="email"
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
