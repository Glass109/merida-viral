mod app;
mod i18n;
mod ui;
mod videos;

#[tokio::main]
async fn main() {
    topcoat::start(app::router()).await.unwrap();
}
