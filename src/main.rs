use library::ui::App;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let _ = app.start().await;
}
