use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::{LocalStorage, Storage};
use web_sys::{HtmlInputElement, HtmlSelectElement};
use shared::{LoginResponse, RegisterResponse};

fn api(path: &str) -> String {
    format!("/api{}", path)
}

#[function_component(RegisterPage)]
pub fn register_page() -> Html {
    let role_ref = use_node_ref();

    let name = use_state(|| "".to_string());
    let email = use_state(|| "".to_string());
    let password = use_state(|| "".to_string());
    let message = use_state(|| "".to_string());

    let on_submit = {
        let role_ref = role_ref.clone();
        let name = name.clone();
        let email = email.clone();
        let password = password.clone();
        let message = message.clone();

        Callback::from(move |_| {
            let message = message.clone();
            let role = role_ref
                .cast::<HtmlSelectElement>()
                .map(|s| s.value())
                .unwrap_or_else(|| "group".to_string());

            let name_val = (*name).clone();
            let email_val = (*email).clone();
            let password_val = (*password).clone();
            message.set(format!("Submitting as role: {}", role));

            spawn_local(async move {
                let register_body = serde_json::json!({
                    "name": name_val,
                    "email": email_val,
                    "password": password_val
                });

                let register_url = if role == "company" {
                    api("/company")
                } else {
                    api("/group")
                };

                let register_req = Request::post(&register_url)
                    .header("Content-Type", "application/json")
                    .body(register_body.to_string())
                    .expect("build register request");

                let register_resp = match register_req.send().await {
                    Ok(r) => r,
                    Err(err) => {
                        message.set(format!("Register error: {:?}", err));
                        return;
                    }
                };

                let register_parsed = match register_resp.json::<RegisterResponse>().await {
                    Ok(p) => p,
                    Err(_) => {
                        message.set("Register parse error".into());
                        return;
                    }
                };

                message.set(register_parsed.message.clone());

                if !register_parsed.ok {
                    return;
                }
                let login_body = serde_json::json!({
                    "email": email_val,
                    "password": password_val
                });

                let login_req = Request::post(&api("/login"))
                    .header("Content-Type", "application/json")
                    .body(login_body.to_string())
                    .expect("build login request");

                let login_resp = match login_req.send().await {
                    Ok(r) => r,
                    Err(err) => {
                        message.set(format!("Login error: {:?}", err));
                        return;
                    }
                };

                let login_parsed = match login_resp.json::<LoginResponse>().await {
                    Ok(p) => p,
                    Err(_) => {
                        message.set("Login parse error".into());
                        return;
                    }
                };

                message.set(login_parsed.message.clone());

                if !login_parsed.ok {
                    return;
                }
                let Some(session) = login_parsed.session_id.clone() else {
                    message.set("Login ok, but no session id".into());
                    return;
                };

                let _ = LocalStorage::set("session_id", session.clone());

                match login_parsed.role.as_deref() {
                    Some("company") => {
                        web_sys::window().unwrap().location()
                            .set_href(&format!("/dashboard/company?session_id={}", session))
                            .unwrap();
                    }
                    Some("group") => {
                        web_sys::window().unwrap().location()
                            .set_href(&format!("/dashboard/group?session_id={}", session))
                            .unwrap();
                    }
                    _ => {
                        message.set("Login ok, but missing role".into());
                    }
                }
            });
        })
    };

    html! {
        <div>
            <h2>{ "Register" }</h2>

            <label>{ "Account type: " }</label>
            <select ref={role_ref}>
                <option value="group">{ "Group" }</option>
                <option value="company">{ "Company" }</option>
            </select>

            <div>
                <input
                    placeholder="name"
                    value={(*name).clone()}
                    oninput={{
                        let name = name.clone();
                        Callback::from(move |e: InputEvent| {
                            let val = e.target_unchecked_into::<HtmlInputElement>().value();
                            name.set(val);
                        })
                    }}
                />
            </div>

            <div>
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
            </div>

            <div>
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
            </div>

            <button onclick={on_submit}>{ "Create account" }</button>

            <p>{ (*message).clone() }</p>
        </div>
    }
}
