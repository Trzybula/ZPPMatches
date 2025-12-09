use yew::prelude::*;
use yew_router::prelude::*;

mod pages;
use pages::{HomePage, LoginGroupPage, LoginCompanyPage, DashboardCompanyPage, DashboardGroupPage, NotFoundPage, MatchPage};

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/login/group")]
    LoginGroup,
    #[at("/login/company")]
    LoginCompany,
    #[at("/dashboard/group")]
    DashboardGroupPage,
    #[at("/dashboard/company")]
    DashboardCompanyPage,
    #[at("/match")]
    MatchPage,
    #[not_found]
    #[at("/404")]
    NotFound,
}


fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <HomePage /> },
        Route::LoginGroup => html! { <LoginGroupPage /> },
        Route::LoginCompany => html! { <LoginCompanyPage /> },
        Route::DashboardGroupPage => html! { <DashboardGroupPage /> },
        Route::DashboardCompanyPage => html! { <DashboardCompanyPage /> },
        Route::NotFound => html! { <NotFoundPage /> },
        Route::MatchPage => html! { <MatchPage /> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
