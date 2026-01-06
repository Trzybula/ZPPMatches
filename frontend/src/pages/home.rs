use yew::prelude::*;
use yew_router::prelude::*;
use crate::Route;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    html! {
        <div>
            <h1>{"Welcome to the ZPPMatch"}</h1>

            <div style="margin: 8px 0;">
                <Link<Route> to={Route::Login}>
                    <button>{"Login"}</button>
                </Link<Route>>
            </div>

            <div style="margin: 8px 0;">
                <Link<Route> to={Route::Register}>
                    <button>{"Register"}</button>
                </Link<Route>>
            </div>
        </div>
    }
}

