use super::*;


#[component]
pub fn PosSize<G: Html>() -> View<G> {

  let (box_ref, _) = pointer_down_move_up_moving(None, None);
  let (box_ref2, _) = pointer_down_move_up_moving(None, None);

  // configures
  let abs_front_x = create_signal(false);
  let abs_outward_x = create_signal(false);
  let abs_front_y = create_signal(false);
  let abs_outward_y = create_signal(false);

  let make_abs_possize = move || {
    AbsPosSize::new(
      (abs_front_x.get(), abs_outward_x.get(), Sizon::abs(10.), 150., 10., 10.),
      (abs_front_y.get(), abs_outward_y.get(), Sizon::abs(10.), 250., 10., 10.)
    )
  };

  let make_abs_possize2 = move || {
    AbsPosSize::new(
      (abs_front_x.get(), abs_outward_x.get(), Sizon::rel(0.5), 150., 10., 10.),
      (abs_front_y.get(), abs_outward_y.get(), Sizon::rel(0.5), 250., 10., 10.)
    )
  };

  // abs
  let abs = create_signal(false);

  let abs_ref = create_node_ref();
  let abs_possize = create_signal(make_abs_possize());
  create_effect(on(abs, move || {

    let possize = abs_possize.get_untracked();
    if let Some(bx) = ref_get::<_, Element>(box_ref) {
      if let Some(x) = ref_get::<_, HtmlElement>(abs_ref) {
        possize.set_style(bx, x)
      }
    }
  }));

  create_effect(on((abs_front_x, abs_outward_x, abs_front_y, abs_outward_y), move || {
    abs_possize.set(make_abs_possize());
  }));

  // abs2
  let abs2 = create_signal(false);

  let abs_ref2 = create_node_ref();
  let abs_possize2 = create_signal(make_abs_possize2());
  create_effect(on(abs2, move || {

    let possize = abs_possize2.get_untracked();
    if let Some(bx) = ref_get::<_, Element>(box_ref2) {
      if let Some(x) = ref_get::<_, HtmlElement>(abs_ref2) {
        possize.set_style(bx, x)
      }
    }
  }));

  create_effect(on((abs_front_x, abs_outward_x, abs_front_y, abs_outward_y), move || {
    abs_possize2.set(make_abs_possize2());
  }));


  // fixed // activate/deactivated by mouse click
  let fixed = create_signal(false);
  
  let fixed_ref = create_node_ref();
  let fixed_possize = FixedPosSize::new(
    (100., 10., 10.),
    (150., 10., 10.)
  );
  let fixed_possize = create_signal(fixed_possize);

  let locate_boxes = move || {
    let Some(box1) = ref_get::<_, HtmlElement>(box_ref) else { return };
    let Some(box2) = ref_get::<_, HtmlElement>(box_ref2) else { return };

    box1.style().set_property("bottom", "200px").unwrap_throw();
    box2.style().set_property("bottom", "200px").unwrap_throw();
  };

  view! {
    h1(style="margin-left: 16px;", id="possize") { "Test PosSize"}

    div(style="margin: 8px 16px;") {
      div() {
        div() { "click somewhere to activate/deactivate fixed menubar (red box)" }
        div() { "drag abs boxes and click them to activate absolute tooltip (blue box)" }
        div() { "See how the menubar/tooltip are positioned according to their activated position"}
      }
      button(on:click=move |_| locate_boxes()) {"locate abs boxes"}
    }

    div(style="margin: 8px 16px;") {
      CheckBox(name="abs_front_x", signal=abs_front_x)
      CheckBox(name="abs_outward_x", signal=abs_outward_x)
      CheckBox(name="abs_front_y", signal=abs_front_y)
      CheckBox(name="abs_outward_y", signal=abs_outward_y)
    }

    div(class="full", on:pointerdown=move |e: PointerEvent| {
      fixed.set(!fixed.get());
      if fixed.get() {
        let xy = (e.client_x() as f64, e.client_y() as f64);
        let possize = fixed_possize.get_untracked();
        if let Some(x) = ref_get::<_, HtmlElement>(fixed_ref) {
          possize.set_style(x, xy);
        }
      }
    }) {
      div(ref=box_ref, class="possize-box", on:pointerdown=move |e: PointerEvent| e.stop_propagation()) {
        button(on:click=move |_| abs.set(!abs.get())) {"abs"}
        (if abs.get() { view! { 
          div(ref=abs_ref, class="possize-abs", on:pointerdown=move |_| abs.set(!abs.get())) {} 
        }} else { view! {}})
      }
      div(ref=box_ref2, class="possize-box", style="left: 200px", on:pointerdown=move |e: PointerEvent| e.stop_propagation()) {
        button(on:click=move |_| abs2.set(!abs2.get())) {"abs2"}
        (if abs2.get() { view! { 
          div(ref=abs_ref2, class="possize-abs", on:pointerdown=move |_| abs2.set(!abs2.get())) {} 
        }} else { view! {}})
      }
      (if fixed.get() { view! { div(ref=fixed_ref, class="possize-fixed") }} else { view! {}})
    }
  }
}