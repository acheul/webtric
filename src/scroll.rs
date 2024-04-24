//! About scroll, we gotta think in combinational way.
//! 
//! ## Scrolling
//! First of all, there is an element which is actually "scrolling".
//! In some cases we would only take care of this.
//! Like when we need just current scroll postion for some other works.
//! 
//! The [`UniScrollMetric`] and [`ScrollMetric`] struct of this module is for this case.
//! They capture current scrolling context: client's size, scroll' size, and scroll' position.
//! 
//! We would like to attach **scroll event** to the scrolling element so as to update the metric value.
//! 
//! Additionally, we might want the scrolling element to consume orthogonal wheeling event:
//! Ex., wheeling horizontally makes the element scroll vertically.
//! We would use **wheel event** for this case.
//! 
//! ## Scrollbar
//! Second, there can be a scrollbar
//! which has "scroll track" and "scroll thumb".
//! 
//! The "scroll track" should be parallel to it's scrolling buddy,
//! and have **pointerdown event** listener which helps navigate scrolling position.
//! (If you click a position of the track, then it will naviagte you to its proportional scroll position.)
//! 
//! The "scroll thumb" is scrolling handle. It would be located under the "scroll track".
//! With **pointerdown event** of it, we would be able to navigate scrolling position.
//! (Movinng scroll thumb will make the srolling element scroll.)
//! 
//! ## [ScrollMetric] for event listeners
//! The ScrollMetric struct has some associated functions
//! to produce event listeners for scrolling element and scrollbar's elements as mentioed above.
//! 
//! They are generalized at some levels,
//! but more applicated ones are designed for **Sycamore**(version 0.9.0-beta.2 or later) with `sycamore` feature on.
//! 
//! # Supposed elements' html structure
//! 
//! Make sure that the scrollbar(scroll track and scroll thumb) should be
//! parallel to the scrolling event.
//! 
//! To make an element scroll, it should have { overflow: scroll } and explicit size.
//! 
//! Below is an example structure where a vertical scrollbar is located at right side of scorlling element.
//! * wrap { position: relative; }
//!   * scrolling { overflow: scroll; height: 200px; width: 200px; }
//!   * track { position: absolute; top: 0; bottom: 0; right: 0; width: 10px; }
//!     * thumb { position: absolute; left: 0; right: 0; }
//! 
//! 
//! Tip: To hide browser's automatic scrollbar,
//! the scrolling element can have css style of something like below.
//! ```css
//! /* Chrome, Safari, Opera */
//! .no-scrollbar::-webkit-scrollbar {
//!   display: none;
//! }
//! .no-scrollbar {
//!   -ms-overflow-style: none;  /* IE and Edge */
//!   scrollbar-width: none;  /* Firefox */
//! }
//! ```


use crate::*;

/// Capture scrolling context. (Uni dimensional)
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UniScrollMetric {
  /// clientWidth/clientHeight
  pub client_size: f64,
  /// scrollWidth/scrollHeight
  pub scroll_size: f64,
  /// scrollTop/scrollLeft
  pub scroll_pos: f64
}


impl UniScrollMetric {

  pub fn scroll_ratio(&self) -> f64 {
    if self.scroll_size.is_normal() { self.scroll_pos/self.scroll_size } else { 0. }
  }

  /// extended scroll ratio, subtracting client size to the scroll size
  pub fn extended_scroll_ratio(&self) -> f64 {
    self.scroll_pos/(self.scroll_size-self.client_size)
  }

  pub fn client_ratio(&self) -> f64 {
    if self.scroll_size.is_normal() { self.client_size/self.scroll_size } else { 0. }
  }

  pub fn scrollable(&self) -> bool {
    let gap = self.scroll_size-self.client_size;
    gap.floor()>0. // Use floor instead of round
  }

  pub fn thumb_pos(&self) -> f64 {
    self.client_size * self.scroll_ratio()
  }

  pub fn thumb_size(&self) -> f64 {
    self.client_size * self.client_ratio()
  }

  /// return metric from given scrolling element.
  /// Use [ScrollMetric] to capture both of x y direction.
  pub fn measures<E: AsRef<Element>>(scrolling: E, lateral: bool) -> Self {

    let elem = scrolling.as_ref();
    
    // <!> For scrolling element's client size, use `clientWidth/Height`, not `getBoundingClientRect`.
    // Because getBoundingClientRect would calculate hidden area as long as it's inside the viewPort.
    // However, clientWidth/Height would only consider visible area (for the case of scrolling elements, which have designated style size).
    let (client_size, scroll_size, scroll_pos) = if lateral {
      (elem.client_width() as f64, elem.scroll_width() as f64, elem.scroll_left() as f64)
    } else {
      (elem.client_height() as f64, elem.scroll_height() as f64, elem.scroll_top() as f64)
    };
  
    Self { client_size, scroll_size, scroll_pos }
  }
}


/// Capture scrolling context both of horizontal and vertical direction.
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScrollMetric {
  pub x: UniScrollMetric,
  pub y: UniScrollMetric
}

impl ScrollMetric {

  /// return metric
  pub fn measures<E: AsRef<Element>>(scrolling: E) -> Self {

    let elem = scrolling.as_ref();

    let (x0, x1, x2) = (elem.client_width() as f64, elem.scroll_width() as f64, elem.scroll_left() as f64);
    let (y0, y1, y2) = (elem.client_height() as f64, elem.scroll_height() as f64, elem.scroll_top() as f64);
    
    Self {
      x: UniScrollMetric { client_size: x0, scroll_size: x1, scroll_pos: x2 },
      y: UniScrollMetric { client_size: y0, scroll_size: y1, scroll_pos: y2 }
    }
  }

  /// manual scroll by given `delta` with given direction(lateral or vertical)
  pub fn scroll_by<E: AsRef<Element>>(scrolling: E, delta: f64, lateral: bool) {
    let (x, y) = if lateral { (delta, 0.) } else { (0., delta) };
    scrolling.as_ref().scroll_by_with_x_and_y(x, y);
  }

  /// manual scroll to given `pos` with given direction(lateral or vertical)
  pub fn scroll_to<E: AsRef<Element>>(scrolling: E, pos: f64, lateral: bool) {
    let (x, y) = if lateral { (pos, 0.) } else { (0., pos) };
    scrolling.as_ref().scroll_to_with_x_and_y(x, y);
  }

  /// Return event listeners closures for "scrolling" element.
  /// * scroll event listener
  /// * (possible) wheel event listener
  /// 
  /// # Args
  /// * x_take_ortho: To consume vertical wheel event to trigger lateral scroll event or not.
  /// * y_take_ortho: To cosume lateral wheel event to trigger vertical scroll event or not.
  /// * scrolling: a wrapping reference of scrolling element, which can be parsed into AsRef<web_sys::Element> with `get_elem`. 
  /// * get_elem: this closure should transforms `scrolling` argument into `AsRef<Element>`
  ///   - We're using `scrolling` and `get_elem` to capture exact target element:
  ///     using `event.eventTarget` would capture wrong target.
  /// * scroll_work: inner closure of scroll event. This is for more detailed application.
  /// 
  /// # Outputs
  /// * Scroll event listener: Closure<dyn FnMut(Event)>
  /// * wheel event listener: Option<Closure<dyn FnMut(WheelEvent)>>
  ///   - If both args `x_take_ortho` and `y_take_ortho` are false, it would be None.
  /// 
  /// # Applications
  /// * *sycamore* => [`init_scrolling()`]
  /// * ~~*letpos* => [`leptos_init_scrolling()`]~~
  pub fn scrolling_listeners<X: Copy + 'static, E: AsRef<Element>>(
    x_take_ortho: bool,
    y_take_ortho: bool,
    scrolling: X,
    get_elem: impl Fn(X) -> Option<E> + Copy + 'static,
    scroll_work: impl Fn(Self) -> () + 'static
  ) -> (
    Closure<dyn FnMut(Event)>, 
    Option<Closure<dyn FnMut(WheelEvent)>>
  ) {
    let cb_scroll: Closure<dyn FnMut(Event)> = Closure::<dyn FnMut(_)>::new(move |_: Event| {
      if let Some(scrolling) = get_elem(scrolling) {
        let metric = ScrollMetric::measures(scrolling);
        let _ = scroll_work(metric);
      }
    });

    let cb_wheel = if x_take_ortho || y_take_ortho {
      let cb: Closure<dyn FnMut(WheelEvent)> = Closure::<dyn FnMut(_)>::new(move |e: WheelEvent| {
        if let Some(scrolling) = get_elem(scrolling) {
          if x_take_ortho {
            scrolling.as_ref().scroll_by_with_x_and_y(e.delta_y(), 0.);
          }
          if y_take_ortho {
            scrolling.as_ref().scroll_by_with_x_and_y(0., e.delta_x());
          }
        }
      });
      Some(cb)
    } else {
      None
    };

    (cb_scroll, cb_wheel)
  }

  /// Expand [`scrolling_listeners()`] for ready made use in Sycamore.
  /// 
  /// Use `init_scrolling_and_scrollbars()` to combine it with scrollbar's work. (recommended way)
  /// 
  /// # Args
  /// * scrolling_ref: NodeRef of scrolling element. 
  /// * scroll_metric: signal of Scrollmetric
  /// * update_by: tuple of signals which can update scroll_metric.
  ///   * Ex. window_resizing signal 
  /// * scroll_x_to: signal which can manually trigger scroll event: horizontally scroll to its value.
  /// * scroll_y_to: signal which can manually trigger scroll event: vertically scroll to its value.
  /// 
  /// # Outputs
  /// (scrolling_ref, scroll_metric, scroll_x_to, scroll_y_to)
  /// 
  /// # Example
  /// ```
  /// # use webtric::*;
  /// # use sycamore::prelude::*;
  /// #[component]
  /// fn Component<G: Html>() -> View<G> {
  ///   
  ///   let window_resizing = WindowResizing::init();
  ///   // let WindowResizing(window_resizing) = use_context();
  /// 
  ///   let (scrolling_ref, scroll_metric, _scroll_x_to, _scroll_y_to) =
  ///     ScrollMetric::init_scrolling(false, false, None, None, *window_resizing, None, None);
  /// 
  ///   view! {
  ///     div(ref=scrolling_ref, style="overflow: scroll; width: 100%; height: 100%;") {
  ///       // ...
  ///     }
  ///   }
  /// }
  /// ```
  /// 
  /// *feature `sycamore`*
  #[cfg(feature="sycamore")]
  pub fn init_scrolling<G: GenericNode, U: Trackable + 'static>(
    x_take_ortho: bool,
    y_take_ortho: bool,
    scrolling_ref: Option<NodeRef<G>>,
    scroll_metric: Option<Signal<Self>>,
    update_by: U,
    scroll_x_to: Option<Signal<f64>>,
    scroll_y_to: Option<Signal<f64>>
  ) -> (NodeRef<G>, Signal<Self>, Signal<f64>, Signal<f64>) {
    
    let scrolling_ref: NodeRef<G> = scrolling_ref.unwrap_or(create_node_ref());
    let scroll_metric = scroll_metric.unwrap_or(create_signal(ScrollMetric::default()));
    let scroll_x_to = scroll_x_to.unwrap_or(create_signal(0.));
    let scroll_y_to = scroll_y_to.unwrap_or(create_signal(0.));

    let scroll_work = move |metric: ScrollMetric| scroll_metric.set(metric);
    let (cb_scroll, cb_wheel) = 
      Self::scrolling_listeners(x_take_ortho, y_take_ortho, scrolling_ref, ref_get::<_, Element>, scroll_work);

    on_mount(move || {
      create_effect(on(update_by, move || {
        ref_get::<_, Element>(scrolling_ref).map(|scrolling| {
          let metric = Self::measures(scrolling);
          scroll_metric.set(metric);
        });
      }));

      create_effect(on(scroll_x_to, move || {
        ref_get::<_, Element>(scrolling_ref).map(|scrolling| {
          let y = scrolling.scroll_top();
          scrolling.scroll_to_with_x_and_y(scroll_x_to.get(), y as f64);
        });
      }));

      create_effect(on(scroll_y_to, move || {
        ref_get::<_, Element>(scrolling_ref).map(|scrolling| {
          let x = scrolling.scroll_left();
          scrolling.scroll_to_with_x_and_y(x as f64, scroll_y_to.get());
        });
      }));

      // set listeners
      ref_get::<_, EventTarget>(scrolling_ref).map(|scrolling| {
        scrolling.add_event_listener_with_callback("scroll", cb_scroll.as_ref().unchecked_ref()).unwrap_throw();
        if let Some(cb_wheel) = cb_wheel.as_ref() {
          scrolling.add_event_listener_with_callback("wheel", cb_wheel.as_ref().unchecked_ref()).unwrap_throw();
        }

        on_cleanup(move || {
          scrolling.remove_event_listener_with_callback("scroll", cb_scroll.as_ref().unchecked_ref()).unwrap_throw();
          if let Some(cb_wheel) = cb_wheel {
            scrolling.remove_event_listener_with_callback("wheel", cb_wheel.as_ref().unchecked_ref()).unwrap_throw();
          }
        });
      });
    });

    (scrolling_ref, scroll_metric, scroll_x_to, scroll_y_to)
  }


  /*/// Check [`init_scrolling()`] of feature *sycamore*
  #[cfg(feature="leptos")]
  pub fn leptos_init_scrolling<T: leptos::html::ElementDescriptor + 'static>(
    x_take_ortho: bool,
    y_take_ortho: bool,
    scrolling_ref: Option<leptos::NodeRef<T>>,
    scroll_metric: Option<leptos::WriteSignal<Self>>,
    update_by: leptos::ReadSignal<bool>,
    scroll_x_to: Option<leptos::WriteSignal<f64>>,
    scroll_y_to: Option<leptos::WriteSignal<f64>>
  ) -> (leptos::NodeRef<T>, leptos::WriteSignal<Self>, leptos::WriteSignal<f64>, leptos::WriteSignal<f64>) {
    use leptos::SignalSet;
    
    let scrolling_ref: leptos::NodeRef<T> = scrolling_ref.unwrap_or(leptos::create_node_ref::<T>());
    let scroll_metric = scroll_metric.unwrap_or(leptos::create_signal(ScrollMetric::default()).1);
    let scroll_x_to = scroll_x_to.unwrap_or(leptos::create_signal(0.).1);
    let scroll_y_to = scroll_y_to.unwrap_or(leptos::create_signal(0.).1);

    let scroll_work = move |metric: ScrollMetric| scroll_metric.set(metric);
    let (cb_scroll, cb_wheel) = 
      Self::scrolling_listeners(x_take_ortho, y_take_ortho, scrolling_ref, ref_get::<_, Element>, scroll_work);

    // TODO
    
    (scrolling_ref, scroll_metric, scroll_x_to, scroll_y_to)
  }*/



  /// Return event listeners closures for scrollbar elements: "scroll track" and "scroll thumb"
  /// * scroll track
  ///   * pointerdown event
  /// * scroll thumb
  ///   * pointerdown event
  ///
  /// # Args
  /// * lateral: Does the bar care about lateral scroll or vertical scroll?
  /// * scrolling: The scrolling element's possible wrapping reference of web_sys::Element
  /// * get_elem: this closure should transforms `scrolling`'s type `<X>` into `AsRef<Element>`
  ///   - We're using this to capture exact target element, which would be failed with `event.EventTarget`.
  /// * thumb_pointerdown_work: inner closure for scroll thumbs' pointerdown event.
  ///   - ex. do something to notify thumb's moving(scorlling) starts.
  /// * thumb_pointerup_work: inner closure for scroll thumbs' (document) pointerup event.
  ///   - ex. do something to notify thumb's moving(scorlling) ends.
  /// 
  /// # Outputs
  /// * track's pointerdown event listener: `Closure<dyn FnMut(PointerEvent)>`
  /// * thumb's pointerdown event listener: `Closure<dyn FnMut(PointerEvent)>`
  /// * raw pointers output(`BoxRaws<(PointerMoveUpBoxRaws, *mut Option<f64>)>`):
  ///   - they are raw pointers generated from thumb's pointerdown event listener closure.
  ///     Clean these raw pointer whenever the thumb's pointerdown event gets cleaned up.
  ///   - To clean them, use `clean()` method of `BoxRaws`. Check out [rawn](https://crates.io/crates/rawn) for more info about `BoxRaws`.
  /// 
  /// # Applications
  /// * *sycamore* => [`init_scrollbar()`]
  /// * ~~*letpos* => [`leptos_init_scrollbar()`]~~
  pub fn scrollbar_listeners<X: Copy + 'static, E: AsRef<Element>>(
    lateral: bool,
    scrolling: X,
    track: X,
    get_elem: impl Fn(X) -> Option<E> + Copy + 'static,
    thumb_pointerdown_work: impl Fn() -> () + 'static,
    thumb_pointerup_work: impl Fn() -> () + 'static
  ) -> (
    Closure<dyn FnMut(PointerEvent)>,
    Closure<dyn FnMut(PointerEvent)>,
    BoxRaws<(PointerMoveUpBoxRaws, *mut Option<f64>)>
  ) {
    // track
    let cb_pointerdown_track: Closure<dyn FnMut(PointerEvent)> = Closure::<dyn FnMut(_)>::new(move |e: PointerEvent| {
      if let Some(track) = get_elem(track) {
        
        let (front, size) = get_elem_front_and_size(&track, lateral);
        let client_pos = if lateral { e.client_x() } else { e.client_y() };
        let d = (client_pos as f64) - front;

        if 0.<d && d<size && size>0. {
          let r = d/size;
          if let Some(scrolling) = get_elem(scrolling) {
            
            // (a) This makes pointerdown position = bar's front(top/left)
            /* let scroll_size = get_scroll_size(scrolling.as_ref(), lateral);
            let to = scroll_size * r; */
            
            // (b) This makes pointerdown position = bar's middle point. It's more ergonomic?
            let (client_size, scroll_size) = 
              if lateral { (scrolling.as_ref().client_width() as f64, scrolling.as_ref().scroll_width() as f64) }
              else { (scrolling.as_ref().client_height() as f64, scrolling.as_ref().scroll_height() as f64) };
            let to = scroll_size * r - client_size * 0.5;

            let (x, y) = if lateral { (to, scrolling.as_ref().scroll_top() as f64) } else { (scrolling.as_ref().scroll_left() as f64, to) };
            scrolling.as_ref().scroll_to_with_x_and_y(x, y);
          }
        }
      }
    });

    // thumb
    let x: *mut Option<f64> = Box::into_raw(Box::new(None::<f64>));

    let pointer_move = move |e: PointerEvent| {
      unsafe {
        let x1 = if lateral { e.client_x() } else { e.client_y() } as f64;
        if let Some(x0) = (*x).replace(x1) {
          let delta = x1-x0;

          if let Some(scrolling) = get_elem(scrolling) {
            let scroll_size = get_scroll_size(scrolling.as_ref(), lateral);
            let client_size = get_client_size(scrolling.as_ref(), lateral);
            let client_ratio = client_size/scroll_size;
            let delta = delta / client_ratio;
            let (x, y) = if lateral { (delta, 0.) } else { (0., delta) };
            scrolling.as_ref().scroll_by_with_x_and_y(x, y);
          }
        }
      }
    };

    let pointer_up = move |_| {
      unsafe { let _ = (*x).take(); }
      thumb_pointerup_work();
    };

    let pointer_down = move |e: PointerEvent| {

      e.stop_propagation();
      unsafe {
        let x1 = if lateral { e.client_x() } else { e.client_y() } as f64;
        let _ = (*x).replace(x1);
      }
      thumb_pointerdown_work();
    };

    let (cb_pointerdown, raws) = pointer_down_move_up(pointer_down, pointer_move, pointer_up);

    (
      cb_pointerdown_track,
      cb_pointerdown,
      BoxRaws((raws, x))
    )
  }

  /// Helper to update thumb's front position and size style property.
  /// the thumb is supposed to be { position: absolute } and relative to its track.
  pub fn update_thumb_style<H: AsRef<HtmlElement>>(&self, thumb: H, lateral: bool) {

    let metric = if lateral { &self.x } else { &self.y };
    let pos = format!("{:.2}%", metric.scroll_ratio()*100.);
    let size = format!("{:.2}%", metric.client_ratio()*100.);
    let (size_prop, pos_prop) = size_pos_props(lateral);

    let style = thumb.as_ref().style();
    style.set_property(pos_prop, pos.as_str()).unwrap_throw();
    style.set_property(size_prop, size.as_str()).unwrap_throw();
  }

  /// Expand [`scrollbar_listeners()`] for ready made use in Sycamore.
  /// 
  /// Also make a `create_effect` on `scroll_metric` so as to update thumb element's style reactively.
  /// 
  /// Use [`init_scrolling_and_scrollbars()`] to combine it with scrolling's work. (recommended way)
  /// 
  /// # Args
  /// * scrolling_ref: NodeRef of scrolling element. Required.
  /// * scroll_metric: signal of Scrollmetric. Required.
  /// * trac_ref: NodeRef of track element.
  /// * thumb_ref: NodeRef of thumb element.
  /// * thumb_moving: signal notifying thumb's moving starts or ends.
  /// 
  /// # Outputs
  /// (track_ref, thumb_ref, thumb_moving)
  /// 
  /// *feature `sycamore`*
  #[cfg(feature="sycamore")]
  pub fn init_scrollbar<G: GenericNode>(
    lateral: bool,
    scrolling_ref: NodeRef<G>,
    scroll_metric: Signal<Self>,
    track_ref: Option<NodeRef<G>>,
    thumb_ref: Option<NodeRef<G>>,
    thumb_moving: Option<Signal<bool>>
  ) -> (NodeRef<G>, NodeRef<G>, Signal<bool>) {

    let track_ref: NodeRef<G> = track_ref.unwrap_or(create_node_ref());
    let thumb_ref: NodeRef<G> = thumb_ref.unwrap_or(create_node_ref());
    let thumb_moving = thumb_moving.unwrap_or(create_signal(false));

    let thumb_pointerdown_work = move || thumb_moving.set(true);
    let thumb_pointerup_work = move || thumb_moving.set(false);

    let (cb_pointerdown_track, cb_pointerdown, raws) = 
      Self::scrollbar_listeners(lateral, scrolling_ref, track_ref, ref_get::<_, Element>, thumb_pointerdown_work, thumb_pointerup_work);

    on_mount(move || {

      create_effect(on(scroll_metric, move || {
        scroll_metric.with(|metric| {
          ref_get::<_, HtmlElement>(thumb_ref).map(|thumb| {
            Self::update_thumb_style(metric, thumb, lateral);
          });
        });
      }));

      ref_get::<_, EventTarget>(track_ref).map(|track: EventTarget| {
        track.add_event_listener_with_callback("pointerdown", cb_pointerdown_track.as_ref().unchecked_ref()).unwrap_throw();
        on_cleanup(move || {
          track.remove_event_listener_with_callback("pointerdown", cb_pointerdown_track.as_ref().unchecked_ref()).unwrap_throw();
        });
      });

      ref_get::<_, EventTarget>(thumb_ref).map(|thumb: EventTarget| {
        thumb.add_event_listener_with_callback("pointerdown", cb_pointerdown.as_ref().unchecked_ref()).unwrap_throw();
        on_cleanup(move || {
          thumb.remove_event_listener_with_callback("pointerdown", cb_pointerdown.as_ref().unchecked_ref()).unwrap_throw();
        });
      });

      on_cleanup(move || {
        raws.clean();
      });
    });

    (track_ref, thumb_ref, thumb_moving)
  }


  /// Combine [`scrolling_listeners()`] and [`scrollbar_listeners()`] with minimal argments.
  /// 
  /// # Args
  /// * x_take_ortho: To consume vertical wheel event to trigger lateral scroll event or not.
  /// * y_take_ortho: To cosume lateral whell event to trigger vertical scroll event or not.
  /// * update_by: tuple of signals which can update scroll_metric.
  ///   * Ex. window_resizing signal 
  /// * bar_x: use lateral scrollbar or not
  /// * bar_y: use vertical scrollbar or not
  /// 
  /// # Outputs:
  /// * scroll_ref,
  /// * scroll_metric signal,
  /// * scroll_x_to signal
  /// * scroll_y_to signal
  /// * Option<(track_ref_x, thumb_ref_x)>
  ///   - is some when `bar_x` is true
  /// * Option<(track_ref_y, thumb_ref_y)>
  ///   - is some when `bar_y` is true
  /// * thumb_moving signal
  /// 
  /// # Example
  /// ```
  /// # use webtric::*;
  /// # use sycamore::prelude::*;
  /// #[component]
  /// fn Component<G: Html>() -> View<G> {
  ///   
  ///   let window_resizing = WindowResizing::init();
  ///   // let WindowResizing(window_resizing) = use_context();
  /// 
  ///   let (scrolling_ref, scroll_metric, _scroll_x_to, _scroll_y_to, _, y, thumb_moving) =
  ///     ScrollMetric::init_scrolling_and_scrollbars(false, false, *window_resizing, false, true);
  /// 
  ///   let (track_ref, thumb_ref) = y.unwrap();
  /// 
  ///   view! {
  ///     // wrap element
  ///     div(style="position: relative; width: 100%; height: 100%;") {
  ///       // scrolling element
  ///       div(ref=scrolling_ref, style="overflow: scroll; width: 100%; height: 100%;") {
  ///         // stuffs of scrolling element
  ///       }
  ///       // scrollbar elements: track, thumb
  ///       div(ref=track_ref, style="position: absolute; top: 0; bottom: 0; right: 0; width: 10px") {
  ///         div(ref=thumb_ref, style="position: absolute; left: 0; right: 0; background-color: orange;") {}
  ///       }
  ///     }
  ///   }
  /// }
  /// ```
  /// 
  /// *feature `sycamore`*
  #[cfg(feature="sycamore")]
  pub fn init_scrolling_and_scrollbars<G: GenericNode, U: Trackable + 'static>(
    x_take_ortho: bool,
    y_take_ortho: bool,
    update_by: U,
    bar_x: bool,
    bar_y: bool
  ) -> (
    NodeRef<G>, Signal<Self>, Signal<f64>, Signal<f64>,
    Option<(NodeRef<G>, NodeRef<G>)>,
    Option<(NodeRef<G>, NodeRef<G>)>,
    Signal<bool>
  ) {
    let (scrolling_ref, scroll_metric, scroll_x_to, scroll_y_to) =
      Self::init_scrolling::<G, _>(x_take_ortho, y_take_ortho, None, None, update_by, None, None);
    
    let thumb_moving = create_signal(false);

    let get_bar_refs = move |lateral: bool| {
      let (track_ref, thumb_ref, _) =
        Self::init_scrollbar(lateral, scrolling_ref, scroll_metric, None, None, Some(thumb_moving));
      (track_ref, thumb_ref)
    };

    let x = if bar_x { Some(get_bar_refs(true)) } else { None };
    let y = if bar_y { Some(get_bar_refs(false)) } else { None };

    (
      scrolling_ref, scroll_metric, scroll_x_to, scroll_y_to,
      x, y, thumb_moving
    )
  }
}