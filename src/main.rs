mod app;
mod config;
mod db;
mod models;
mod theme;
mod ui;

fn main() -> iced::Result {
    app::run()
}
