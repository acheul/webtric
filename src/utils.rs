//! Utility helpers of webtric

use crate::*;

/// NewType wrapping Signal<bool>, which would be listening to window's resize event.
/// 
/// It can be used to update window-size related environments, like custom scrollbar or sizing parallel panels.
/// 
/// *feature `sycamore`*
#[cfg(feature="sycamore")]
#[derive(Clone)]
pub struct WindowResizing(pub Signal<bool>);

#[cfg(feature="sycamore")]
impl WindowResizing {
  /// Return a Signal<bool> listening to window's resize event.
  /// 
  /// The NewType wrapping the signal got `provide_context` inside the function.
  /// 
  /// # Example
  /// ```
  /// # use sycamore::prelude::*;
  /// # use webtric::WindowResizing;
  /// # fn App<G: Html>() -> View<G> {
  ///   // initiate it, usually at a root level of the app
  ///   let window_resizing = WindowResizing::init();
  /// 
  ///   // use context of it later
  ///   let WindowResizing(window_resizing) = use_context();
  /// # view! {}
  /// # }
  /// ```
  pub fn init() -> Signal<bool> {

    let signal = create_signal(false);
    let cb_resize = Closure::<dyn FnMut(_)>::new(move |_: Event| {
      signal.set(true);
    });
    
    gloo_utils::window().add_event_listener_with_callback("resize", cb_resize.as_ref().unchecked_ref()).unwrap_throw();
    cb_resize.forget();

    provide_context(Self(signal));
    signal
  }
}



/// Check [WindowResizing] of feature *sycamore*
/// 
/// *feature `leptos`*
#[cfg(feature="leptos")]
#[derive(Clone)]
pub struct LeptosWindowResizing(pub leptos::ReadSignal<bool>);

#[cfg(feature="leptos")]
impl LeptosWindowResizing {
  pub fn init() -> leptos::ReadSignal<bool> {
    use leptos::SignalSet;

    let (signal, set_signal) = leptos::create_signal(false);

    let cb_resize = Closure::<dyn FnMut(_)>::new(move |_: Event| {
      set_signal.set(true);
    });
    
    gloo_utils::window().add_event_listener_with_callback("resize", cb_resize.as_ref().unchecked_ref()).unwrap_throw();
    cb_resize.forget();

    leptos::provide_context(Self(signal));
    signal
  }
}


/// Helper to add or remove a class to NodeRef's element
/// 
/// *feature `sycamore`*
#[cfg(feature="sycamore")]
pub fn alter_class<G: GenericNode>(node: NodeRef<G>, class: &str, add: bool) {
  node.try_get::<DomNode>().map(|node| {
    if add {
      node.add_class(class);
    } else {
      node.remove_class(class);
    }
  });
}


/// Helper to turn NodeRef into web_sys::Element like things.
/// 
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// # use webtric::utils::ref_get;
/// # fn Component<G: Html>() -> View<G> {
/// let node_ref: NodeRef<G> = create_node_ref();
/// 
/// if let Some(element) = ref_get::<_, web_sys::HtmlElement>(node_ref) {
///   let style = element.style();
///   // ... then update style here
/// }
/// 
/// // This equals to
/// if let Some(element) = node_ref.try_get::<DomNode>().map(|x| x.unchecked_into::<web_sys::HtmlElement>()) {
///   let style = element.style();
///   // ...
/// }
/// # view! {}
/// # }
/// ```
/// 
/// *feature `sycamore`*
#[cfg(feature="sycamore")]
pub fn ref_get<G: GenericNode, T: wasm_bindgen::JsCast>(rf: NodeRef<G>) -> Option<T> {
  rf.try_get::<DomNode>().map(|x| x.unchecked_into::<T>())
}


// websys

/// Get ones parent element, or document element as a fallback
pub fn get_par_elem<E: AsRef<Element>>(elem: E) -> Element {
  elem.as_ref().parent_element().unwrap_or(gloo_utils::document_element())
}

/// Get client size: lateral(true/false) -> (clientWidth/clientHeight);
pub fn get_client_size<E: AsRef<Element>>(elem: E, lateral: bool) -> f64 {
  if lateral {
    elem.as_ref().client_width() as f64
  } else {
    elem.as_ref().client_height() as f64
  }
}

/// Get scroll size: lateral(true/false) -> (scrollWidth/scrollHeight);
pub fn get_scroll_size<E: AsRef<Element>>(elem: E, lateral: bool) -> f64 {
  if lateral {
    elem.as_ref().scroll_width() as f64
  } else {
    elem.as_ref().scroll_height() as f64
  }
}

/// Get client size and scroll size
pub fn get_client_scroll_size<E: AsRef<Element>>(elem: E, lateral: bool) -> (f64, f64) {
  if lateral {
    (elem.as_ref().client_width() as f64, elem.as_ref().scroll_width() as f64)
  } else {
    (elem.as_ref().client_height() as f64, elem.as_ref().scroll_height() as f64)
  }
}

// DomRect (getBoundingClientRect)

/// Get DomRect's top, height, left, width
pub fn get_rect_thlw<E: AsRef<Element>>(elem: E) -> (f64, f64, f64, f64) {
  let rect = elem.as_ref().get_bounding_client_rect();
  (rect.top(), rect.height(), rect.left(), rect.width())
}

/// Get DomRect's size
pub fn get_elem_size<E: AsRef<Element>>(elem: E, lateral: bool) -> f64 {
  let rect = elem.as_ref().get_bounding_client_rect();
  if lateral { rect.width() } else { rect.height() }
}

/// Get DomRect's front
pub fn get_elem_front<E: AsRef<Element>>(elem: E, lateral: bool) -> f64 {
  let rect = elem.as_ref().get_bounding_client_rect();
  if lateral { rect.left() } else { rect.top() }
}

/// Get DomRect's front and size:
/// lateral(true/false) -> (left, width)/(top, height)
pub fn get_elem_front_and_size<E: AsRef<Element>>(elem: E, lateral: bool) -> (f64, f64) {
  let rect = elem.as_ref().get_bounding_client_rect();
  if lateral {
    (rect.left(), rect.width())
  } else {
    (rect.top(), rect.height())
  }
}


/// Style properties of size and front position:
/// lateral(true/false) -> ("width", "left")/("height", "top")
pub fn size_pos_props<'a>(lateral: bool) -> (&'a str, &'a str) {
  if lateral { ("width", "left") } else { ("height", "top") }
}

/// Retrieves a dataset value matching given dataset name.
/// It tries to parse String value into generic `<T>` type. 
pub fn parse_dataset<H: AsRef<HtmlElement>, T: FromStr>(elem: H, name: &str) -> Option<T> {
  elem.as_ref().dataset().get(name)
    .map(|value| value.parse().ok()).flatten()
}



/// alias of BoxRaws-wrapping of raw pointers of PointerMove & PointerUp event listeners
pub type PointerMoveUpBoxRaws = BoxRaws<(*mut Closure<dyn FnMut(PointerEvent)>, *mut Closure<dyn FnMut(PointerEvent)>)>;

/// Helper of building pointerEvents's listener closures.
/// 
/// When an element gets dragged with pointer(mouse/touce) movement, like in cases of custom scrollbar or resizing handler,
/// Three event would work in harmony:
///   * On a `pointerdown` event triggered on the element, `pointermove` and `pointerup` event listeners are added to the document.
///   * Then do relevant things while `pointermove` event happens.
///   * Then on a `pointerup` event triggered on the document, `pointermove` and `pointerup`'s listeners are removed from the document.
/// 
/// Thus, only `pointerdown` event listeners will be explicitly added to and removed from the relevant element.
/// 
/// # Use
/// * For arguments, pass inner closures for each event listeners.
/// * This returns 
///   * `pointerdown` event listener
///   * [PointerMoveUpBoxRaws]: this is a wrapping of raw pointers of `pointermove` and `pointerup` listeners
/// * To use outputs:
///   * Attach and detach `pointerdown` listener to the relevant element.
///   * Mind that returned [PointerMoveUpBoxRaws] has raw pointers. Make sure to destruct their memory on any clean up scenario.
///     To destruct them, use `clean()` method of `BoxRaws`. Check out [rawn](https://crates.io/crates/rawn) for more info about `BoxRaws`.
/// 
/// # Example
/// ```
/// # use wasm_bindgen::prelude::*;
/// # use webtric::{*, utils::pointer_down_move_up};
/// # fn demo(element: web_sys::Element) -> () {
/// 
/// let down_work = move |_| { };
/// 
/// let move_work = move |e: web_sys::PointerEvent| {
///   let (x, y) = (e.client_x(), e.client_y());
///   // do something
/// };
/// 
/// let up_work = move |_| { };
/// 
/// let (cb_down, raws) = pointer_down_move_up(down_work, move_work, up_work);
/// 
/// // add pointerdown listener to element
/// element.add_event_listener_with_callback("pointerdown", cb_down.as_ref().unchecked_ref()).unwrap_throw();
/// 
/// // whenever the listeners are not required any more, clean them up
/// // Ex. Within a sycamore's Component, this will be inside the `clean_up` scope.
/// element.remove_event_listener_with_callback("pointerdown", cb_down.as_ref().unchecked_ref()).unwrap_throw();
/// unsafe {
///   raws.clean();
/// }
/// # }
/// ```
pub fn pointer_down_move_up(
  down_work: impl Fn(PointerEvent) -> () + 'static,
  move_work: impl Fn(PointerEvent) -> () + 'static,
  up_work: impl Fn(PointerEvent) -> () + 'static
) -> (
  Closure<dyn FnMut(PointerEvent)>,
  PointerMoveUpBoxRaws
  )
{
  let cb_move = Closure::<dyn FnMut(_)>::new(move |e: PointerEvent| {
    let _ = move_work(e);
  });

  let cb_move = Box::into_raw(Box::new(cb_move));

  let cb_up: *mut Closure<dyn FnMut(PointerEvent)> = Box::into_raw(Box::new(Closure::<dyn FnMut(_)>::new(move |_: PointerEvent| {})));

  unsafe {
    *cb_up = Closure::<dyn FnMut(_)>::new(move |e: PointerEvent| {
      let _ = up_work(e);
      let document = gloo_utils::document();
      document.remove_event_listener_with_callback("pointermove", (*cb_move).as_ref().unchecked_ref()).unwrap_throw();
      document.remove_event_listener_with_callback("pointerup", (*cb_up).as_ref().unchecked_ref()).unwrap_throw();
    });
  }
  
  // pointerdown
  let cb_down = Closure::<dyn FnMut(_)>::new(move |e: PointerEvent| {
    unsafe {
      let document = gloo_utils::document();
      document.add_event_listener_with_callback("pointermove", (*cb_move).as_ref().unchecked_ref()).unwrap_throw();
      document.add_event_listener_with_callback("pointerup", (*cb_up).as_ref().unchecked_ref()).unwrap_throw();
    }
    let _ = down_work(e);
  });

  (cb_down, BoxRaws::new((cb_move, cb_up)))
}


/// Expand `pointer_down_move_up` to make an element move with pointer's movement.
/// 
/// *feature `sycamore`*
#[cfg(feature="sycamore")]
pub fn pointer_down_move_up_moving<G: GenericNode>(
  rf: Option<NodeRef<G>>,
  moving: Option<Signal<bool>>,
) -> (NodeRef<G>, Signal<bool>) {

  let rf = rf.unwrap_or(create_node_ref());
  let moving = moving.unwrap_or(create_signal(false));
  let mut shift_xy: Option<(f64, f64)> = None;
  let shift_xy: *mut Option<(f64, f64)> = &mut shift_xy;
  
  let down_work = move |e: PointerEvent| {
    unsafe {
      let (x, y) = (e.client_x() as f64, e.client_y() as f64);
      if let Some(elem) = ref_get::<_, Element>(rf) {
        let rect = elem.get_bounding_client_rect();
        let shift_xy_ = (x-rect.left(), y-rect.top());
        let _ =(*shift_xy).replace(shift_xy_);
      }
    }
    moving.set(true);
  };

  // * Do not use eventTaget: it would capture wrong target.
  let move_work = move |e: PointerEvent| {
    unsafe {
      if let Some((shift_x, shift_y)) = *shift_xy {
        if let Some(elem) = ref_get::<_, HtmlElement>(rf) {
          let (x, y) = (e.client_x() as f64, e.client_y() as f64);
          let left = x - shift_x;
          let top = y - shift_y;

          let style: web_sys::CssStyleDeclaration = elem.style();
          style.set_property("left", format!("{:.2}px", left).as_str()).unwrap_throw();
          style.set_property("top", format!("{:.2}px", top).as_str()).unwrap_throw();
        }
      }
    }
  };

  let up_work = move |_: PointerEvent| {
    unsafe {
      let _ = (*shift_xy).take();
    }
    moving.set(false);
  };

  let (cb_down, raws) = pointer_down_move_up(down_work, move_work, up_work);

  on_mount(move || {
    if let Some(x) = ref_get::<_, EventTarget>(rf) {
      x.add_event_listener_with_callback("pointerdown", cb_down.as_ref().unchecked_ref()).unwrap_throw();
      on_cleanup(move || {
        x.remove_event_listener_with_callback("pointerdown", cb_down.as_ref().unchecked_ref()).unwrap_throw();
      });
    }
  });
  on_cleanup(move || {
    raws.clean();
  });

  (rf, moving)
}