use yew::prelude::*;
use yew_router::prelude::*;
use crate::Route;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    html! {
        <div>
            <h1>{"Welcome to the ZPPMatch"}</h1>
            <div>
                <Link<Route> to={Route::LoginGroup}>
                    <button>{"Login as Group"}</button>
                </Link<Route>>
            </div>
            <div>
                <Link<Route> to={Route::LoginCompany}>
                    <button>{"Login as Company"}</button>
                </Link<Route>>
            </div>
        </div>
    }
}
