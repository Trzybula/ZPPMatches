use yew::prelude::*;
use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew_router::prelude::*;
use yew_router::AnyRoute;

#[derive(Deserialize, Debug, Clone)]
pub struct LoginResponse {
    pub ok: bool,
    pub message: String,
    pub session_id: Option<String>,
}

#[function_component(LoginGroupPage)]
pub fn login_group_page() -> Html {
    let email = use_state(|| "".to_string());
    let password = use_state(|| "".to_string());
    let message = use_state(|| "".to_string());
    let navigator = use_navigator().unwrap();

    let on_submit = {
        let email = email.clone();
        let password = password.clone();
        let message = message.clone();
        let navigator = navigator.clone();

        Callback::from(move |_| {
            let email_val = (*email).clone();
            let password_val = (*password).clone();
            let message = message.clone();
            let navigator = navigator.clone();

            spawn_local(async move {
                let body = serde_json::json!({
                    "email": email_val,
                    "password": password_val
                });

                let req = Request::post("http://localhost:3000/login/group")
                    .header("Content-Type", "application/json")
                    .body(body.to_string())
                    .expect("Failed to build request");

                match req.send().await {
                    Ok(resp) => {
                        match resp.json::<LoginResponse>().await {
                            Ok(parsed) => {
                                message.set(parsed.message.clone());

                                if parsed.ok {
                                    if let Some(session) = parsed.session_id {
                                        let url = format!("/dashboard/group?session_id={}", session);

                                        navigator.push(&AnyRoute::new(url));
                                    }
                                }
                            }
                            Err(_) => message.set("Parse error".into()),
                        }
                    }
                    Err(err) => message.set(format!("Error: {:?}", err)),
                }
            });
        })
    };

    html! {
        <div>
            <h2>{ "Login (Group)" }</h2>

            <input
                placeholder="email"
                oninput={{
                    let email = email.clone();
                    Callback::from(move |e: InputEvent| {
                        let v = e.target_unchecked_into::<HtmlInputElement>().value();
                        email.set(v);
                    })
                }}
            />

            <input
                type="password"
                placeholder="password"
                oninput={{
                    let password = password.clone();
                    Callback::from(move |e: InputEvent| {
                        let v = e.target_unchecked_into::<HtmlInputElement>().value();
                        password.set(v);
                    })
                }}
            />

            <button onclick={on_submit}>{ "Log in" }</button>

            <p>{ (*message).clone() }</p>
        </div>
    }
}
