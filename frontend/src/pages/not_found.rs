use yew::prelude::*;

#[function_component(NotFoundPage)]
pub fn not_found_page() -> Html {
    html! {
        <div>
            <h1>{"404 - Page Not Found"}</h1>
            <p>{"The page you are looking for does not exist."}</p>
        </div>
    }
}