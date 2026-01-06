use yew::prelude::*;
use yew_router::prelude::*;

mod pages;
use pages::{
    HomePage, LoginPage, RegisterPage,
    DashboardCompanyPage, DashboardGroupPage, NotFoundPage, MatchPage, AdminPage
};

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/")]
    Home,

    #[at("/login")]
    Login,

    #[at("/register")]
    Register,

    #[at("/dashboard/group")]
    DashboardGroupPage,

    #[at("/dashboard/company")]
    DashboardCompanyPage,

    #[at("/match")]
    MatchPage,

    #[not_found]
    #[at("/404")]
    NotFound,

    #[at("/admin")]
    Admin,
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <HomePage /> },
        Route::Login => html! { <LoginPage /> },
        Route::Register => html! { <RegisterPage /> },
        Route::DashboardGroupPage => html! { <DashboardGroupPage /> },
        Route::DashboardCompanyPage => html! { <DashboardCompanyPage /> },
        Route::MatchPage => html! { <MatchPage /> },
        Route::NotFound => html! { <NotFoundPage /> },
        Route::Admin => html! { <AdminPage /> },
    }
}

#[function_component(App)]
fn app() -> Html {
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
