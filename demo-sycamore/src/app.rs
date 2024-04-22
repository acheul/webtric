use web_sys::{Element, HtmlElement, PointerEvent};
use webtric::*;
use webtric::utils::*;
use sycamore_router::{Route, Router, HistoryIntegration};
use sycamore::prelude::*;
#[allow(unused_imports)]
use gloo_console::log;

mod scroll;
mod cartons;
mod possize;


#[derive(Clone, Route)]
pub enum Routes {
  #[to("/")] Index,
  #[to("/scroll")] Scroll,
  #[to("/cartons")] Cartons,
  #[to("/possize")] PosSize,
  #[not_found] NotFound
}

fn switch<G: Html>(route: ReadSignal<Routes>) -> View<G> {
  
  let view = create_memo(on(route, move || match route.get_clone() {
    Routes::Index => view! { Index },
    Routes::Scroll => view! { scroll::Scroll },
    Routes::Cartons => view! { cartons::Cartons },
    Routes::PosSize => view! { possize::PosSize },
    Routes::NotFound => view! { "Not Found" },
  }));
  
  view! { (view.get_clone()) }
}

#[component]
pub fn App<G: Html>() -> View<G> {

  let _window_resizing = WindowResizing::init();

  view! {
    main() {
      Router(
        integration=HistoryIntegration::new(),
        view=switch
      )
    }
  }
}

#[component(inline_props)]
pub fn NavLinks<G: Html>(list: Vec<(&'static str, &'static str)>) -> View<G> {

  view! {
    div(class="nav-links") {
      Keyed(
        iterable=*create_signal(list),
        view=move |(href, title)| view! {
          div(class="nav-link") {
            a(href=href, title=title) { (title) }
          }
        },
        key=|x| *x
      )
    }
  }
}



#[component(inline_props)]
pub fn Index<G: Html>(children: Children<G>) -> View<G> {

  let WindowResizing(window_resizing) = use_context();

  let (scrolling_ref, scroll_metric, _scroll_x_to, _scroll_y_to, x, y, thumb_moving) =
    ScrollMetric::init_scrolling_and_scrollbars(false, false, *window_resizing, true, true);
  let (x_track_ref, x_thumb_ref) = x.unwrap();
  let (y_track_ref, y_thumb_ref) = y.unwrap();

  on_mount(move || {
    create_effect(on(thumb_moving, move || {
      alter_class(scrolling_ref, "select-none", thumb_moving.get());
    }));
    create_effect(on(scroll_metric, move || {
      let (x, y) = scroll_metric.with(|metric| (metric.x.scrollable(), metric.y.scrollable()));
      alter_class(x_track_ref, "opacity0", !x);
      alter_class(x_thumb_ref, "opacity0", !x);
      alter_class(y_track_ref, "opacity0", !y);
      alter_class(y_thumb_ref, "opacity0", !y);
    }));
  });


  let children: View<G> = children.call();

  view! {
    div(class="full scroll-wrap") {
      
      div(ref=x_track_ref, class="scroll-track-x") {
        div(ref=x_thumb_ref, class="scroll-thumb-x", style="background-color: lightgrey;") {}
      }
      div(ref=y_track_ref, class="scroll-track-y") {
        div(ref=y_thumb_ref, class="scroll-thumb-y", style="background-color: lightgrey;") {}
      }

      div(ref=scrolling_ref, class="full no-scrollbar scrolling") {
        div(style="margin: 16px;") {
          h1() {"webtric=web+metric"}
          NavLinks(list=vec![
            ("/", "index"),
            ("/scroll", "test scroll"),
            ("/cartons", "test cartons"),
            ("/possize", "test possize")
          ])
        }
        br()
        (children)
      }
    }
  }
}


#[component(inline_props)]
fn CheckBox<G: Html>(
  name: &'static str,
  signal: Signal<bool>
) -> View<G> {

  let (_, id) = create_unique_id();

  view! {
    div() { 
      label(for=id) { (format!("{}: {}", name, signal.get())) }
      input(type="checkbox", bind:checked=signal, id=id) 
    }
  }
} 


fn new_id(list: &Vec<usize>) -> usize {
  (0..list.len()+1).rev().find(|x| !list.contains(x)).unwrap()
}

fn push_carton(cartons: Signal<Vec<usize>>, updates: Vec<Signal<bool>>, complex: Option<Signal<CartonsComplex<usize>>>) {
  
  let new_id = cartons.with(|x| new_id(x));
  if let Some(complex) = complex {
    complex.update(|complex| {
      if !complex.metric.map.contains_key(&new_id) {
        let x = complex.min.default.abs.unwrap_or_default();
        complex.metric.map.insert(new_id, Some(Sizon::abs(x)));
      }
    });
  }
  
  cartons.update(|cartons| {
    if let Some(x) = (0..cartons.len()+1).rev().find(|x| !cartons.contains(x)) {
      cartons.push(x);
    }
  });
  updates.iter().for_each(|update| update.set(true));
}

fn pop_carton(cartons: Signal<Vec<usize>>, updates: Vec<Signal<bool>>) {
  let b = cartons.update(|cartons| {
    if cartons.len()>0 {
      let _ = cartons.remove(0);
      true
    } else {
      false
    }
  });
  if b {
    updates.iter().for_each(|update| update.set(true));
  }
}

fn d2rgb(d: usize) -> String {

  let d = (d+10)*8;
  let r = d%256;
  let g = (d+100)%256;
  let b = (d+200)%256;

  format!("rgb({}, {}, {})", r, g, b)
}