use axum_macros::debug_handler;

#[debug_handler]
async fn handler<T>() {}

fn main() {}
