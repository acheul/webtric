use web_sys::{Element, HtmlElement, PointerEvent};
use webtric::*;
use webtric::utils::*;
use sycamore::prelude::*;
use wasm_bindgen::prelude::*;
#[allow(unused_imports)]
use gloo_console::log;

mod scroll;
mod cartons;
mod possize;


#[component]
pub fn App<G: Html>() -> View<G> {

  let _window_resizing = WindowResizing::init();

  view! {
    main() {
      Index {
        scroll::Scroll
        br()
        cartons::Cartons
        br()
        possize::PosSize
      }
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
          div(style="display: flex; align-items: center;") {
            a(href="https://crates.io/crates/webtric", title="crate") {
              img(src="https://img.shields.io/crates/v/webtric")
            }
            a(style="margin-left: 8px;", href="https://docs.rs/webtric", title="docs") {
              img(src="https://img.shields.io/docsrs/webtric?label=docs.rs")
            }
            a(href="https://github.com/acheul/webtric", title="repository") {
              svg(style="width: 1rem; margin-left: 8px;", xmlns="http://www.w3.org/2000/svg", viewBox="0 0 496 512", data-src="Font Awesome Free 6.5.2 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free Copyright 2024 Fonticons, Inc.") {
                path(d="M165.9 397.4c0 2-2.3 3.6-5.2 3.6-3.3 .3-5.6-1.3-5.6-3.6 0-2 2.3-3.6 5.2-3.6 3-.3 5.6 1.3 5.6 3.6zm-31.1-4.5c-.7 2 1.3 4.3 4.3 4.9 2.6 1 5.6 0 6.2-2s-1.3-4.3-4.3-5.2c-2.6-.7-5.5 .3-6.2 2.3zm44.2-1.7c-2.9 .7-4.9 2.6-4.6 4.9 .3 2 2.9 3.3 5.9 2.6 2.9-.7 4.9-2.6 4.6-4.6-.3-1.9-3-3.2-5.9-2.9zM244.8 8C106.1 8 0 113.3 0 252c0 110.9 69.8 205.8 169.5 239.2 12.8 2.3 17.3-5.6 17.3-12.1 0-6.2-.3-40.4-.3-61.4 0 0-70 15-84.7-29.8 0 0-11.4-29.1-27.8-36.6 0 0-22.9-15.7 1.6-15.4 0 0 24.9 2 38.6 25.8 21.9 38.6 58.6 27.5 72.9 20.9 2.3-16 8.8-27.1 16-33.7-55.9-6.2-112.3-14.3-112.3-110.5 0-27.5 7.6-41.3 23.6-58.9-2.6-6.5-11.1-33.3 2.6-67.9 20.9-6.5 69 27 69 27 20-5.6 41.5-8.5 62.8-8.5s42.8 2.9 62.8 8.5c0 0 48.1-33.6 69-27 13.7 34.7 5.2 61.4 2.6 67.9 16 17.7 25.8 31.5 25.8 58.9 0 96.5-58.9 104.2-114.8 110.5 9.2 7.9 17 22.9 17 46.4 0 33.7-.3 75.4-.3 83.6 0 6.5 4.6 14.4 17.3 12.1C428.2 457.8 496 362.9 496 252 496 113.3 383.5 8 244.8 8zM97.2 352.9c-1.3 1-1 3.3 .7 5.2 1.6 1.6 3.9 2.3 5.2 1 1.3-1 1-3.3-.7-5.2-1.6-1.6-3.9-2.3-5.2-1zm-10.8-8.1c-.7 1.3 .3 2.9 2.3 3.9 1.6 1 3.6 .7 4.3-.7 .7-1.3-.3-2.9-2.3-3.9-2-.6-3.6-.3-4.3 .7zm32.4 35.6c-1.6 1.3-1 4.3 1.3 6.2 2.3 2.3 5.2 2.6 6.5 1 1.3-1.3 .7-4.3-1.3-6.2-2.2-2.3-5.2-2.6-6.5-1zm-11.4-14.7c-1.6 1-1.6 3.6 0 5.9 1.6 2.3 4.3 3.3 5.6 2.3 1.6-1.3 1.6-3.9 0-6.2-1.4-2.3-4-3.3-5.6-2z")
              }
            }
          }          
          NavLinks(list=vec![
            ("#scroll", "test scroll"),
            ("#cartons", "test cartons"),
            ("#possize", "test possize")
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