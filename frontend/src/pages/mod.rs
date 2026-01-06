pub mod home;
pub mod login;
pub mod register;
pub mod dashboard_company;
pub mod dashboard_group;
pub mod match_page;
pub mod not_found;

pub use home::HomePage;
pub use login::LoginPage;
pub use register::RegisterPage;
pub use dashboard_company::DashboardCompanyPage;
pub use dashboard_group::DashboardGroupPage;
pub use match_page::MatchPage;
pub use not_found::NotFoundPage;
pub mod admin;
pub use admin::AdminPage;