use leptos::*;
use gloo_console::log;
use webtric::*;

fn main() {
  leptos::mount_to_body(|| view! { <App/> })
}

#[component]
fn App() -> impl IntoView {

  let window_resizing = LeptosWindowResizing::init();

  create_effect(move |_| {
    log!(window_resizing.get());
  });

  view! {

  }
}