use super::*;


#[component]
pub fn Cartons<G: Html>() -> View<G> {

  // configures
  let lateral = create_signal(false);
  let independent = create_signal(false);
  let update_by = create_signal(false);

  let initial_metric = 
    vec![(0, Some(Sizon::rel(0.2))), (1, Some(Sizon::rel(0.3))), (2, None), (3, Some(Sizon::rel(0.4)))];
  let min = vec![(2, Sizon::abs(150.))];
  let max = vec![(2, Sizon::rel(0.5))];
  let default_min = Sizon::abs(100.);
  let default_max = Sizon::rel(1.0);
  let allow_zero = vec![(0, true), (2, true)];
  let zeroed_when = Sizon::new(Some(40.), Some(0.5));
  let zeroed_cache = (vec![], 0.1);

  let complex = CartonsComplex::new(
    lateral.get(), independent.get(), None,
    initial_metric.into(), (min, default_min).into(), (max, default_max).into(), 
    (allow_zero, false).into(), (vec![], zeroed_when).into(), zeroed_cache.into()
  );
  let complex: Signal<CartonsComplex<usize>> = create_signal(complex);

  let cartons: Signal<Vec<usize>> = create_signal((0..4).collect());
  let terminal_carton = create_selector(on(cartons, move || {
    cartons.with(|x| {
      x.last().map(|x| *x)
    })
  }));

  let mut passive = complex.get_clone();
  passive.name = "passive";
  let passive = create_signal(passive);
  create_effect(on(complex, move || {
    passive.update(|x| {
      let (metric, lateral) = complex.with(|x| (x.metric.clone(), x.lateral));
      x.metric = metric;
      x.lateral = lateral;
    });
  }));

  let update_scroll = create_signal(false);
  
  let demo = create_memo(on((lateral, independent), move || {

    complex.update(|x| {
      x.lateral = lateral.get();
      x.independent = independent.get();
    });
    update_by.set(true);
    
    view! {
      CartonsDemo(
        lateral=lateral.get(),
        complex=complex,
        passive=passive,
        update_scroll=update_scroll,
        update_by=update_by,
        cartons=cartons,
        terminal_carton=terminal_carton
      )
    }
  }));

  view! {
    Index {
      h1(style="margin-left: 16px;") { "Test Cartons"}
      div(style="margin: 8px 16px;") {
        div() { label(for="lateral") { "lateral: " (if lateral.get() {"horizontal"} else {"vertical"}) }
        input(type="checkbox", bind:checked=lateral, id="lateral") }
  
        div() { label(for="independent") { "resize independently: " (independent.get()) }
        input(type="checkbox", bind:checked=independent, id="independent") }
      }
  
      div(style="margin: 8px 6px;") {
        button(style="margin-right: 8px;", on:click=move |_| { push_carton(cartons, vec![update_by, update_scroll], Some(complex)) }) {"+ push to last"}
        button(on:click=move |_| { pop_carton(cartons, vec![update_by, update_scroll]) }) {"- pop up first"}
      }

      (demo.get_clone())
    }
  }
}


#[component(inline_props)]
pub fn CartonsDemo<G: Html>(
  lateral: bool,
  complex: Signal<CartonsComplex<usize>>,
  passive: Signal<CartonsComplex<usize>>,
  update_scroll: Signal<bool>,
  update_by: Signal<bool>,
  cartons: Signal<Vec<usize>>,
  terminal_carton: ReadSignal<Option<usize>>
) -> View<G> {

  let WindowResizing(window_resizing) = use_context();

  // scrollbar
  let (wrap_ref, scroll_metric, _scroll_x_to, _scroll_y_to, x, y, thumb_moving) =
    ScrollMetric::init_scrolling_and_scrollbars(false, false, (*window_resizing, *update_scroll), lateral, !lateral);
  
  let (track_ref, thumb_ref) = if lateral { x.unwrap() } else { y.unwrap() };

  on_mount(move || {
    create_effect(on(thumb_moving, move || {
      alter_class(wrap_ref, "select-none", thumb_moving.get());
    }));
    create_effect(on(scroll_metric, move || {
      let b = scroll_metric.with(|metric| if lateral { metric.x.scrollable() } else { metric.y.scrollable() });
      alter_class(track_ref, "opacity0", !b);
      alter_class(thumb_ref, "opacity0", !b);
    }));
  });

  // cartons complex wrap
  let _ = CartonsComplex::init_passive_wrap(passive, Some(wrap_ref));
  let _ = CartonsComplex::init_wrap(complex, Some(wrap_ref), (*window_resizing, *update_by));

  // swith zero
  let make_zero = move |x: usize| {
    move |_| {
      complex.update(|complex| {
        if let Some(wrap) = ref_get::<_, Element>(wrap_ref) {
          let _ = complex.switch_zero(wrap, &x, false);
        }
      });
    }
  };

  let zero_restore = move |x: usize| {
    move |_| {
      complex.update(|complex| {
        if let Some(wrap) = ref_get::<_, Element>(wrap_ref) {
          let _ = complex.switch_zero(wrap, &x, true);
        }
      });
    }
  };

  view! {
    div() {
      div(style="margin: 8px 6px;") {
        Keyed(
          iterable=*cartons,
          view=move |x| view! {
            button(on:click=make_zero(x)) {"-" (x)}
          },
          key=|x| *x,
        )
        Keyed(
          iterable=*cartons,
          view=move |x| view! {
            button(on:click=zero_restore(x)) {"+" (x)}
          },
          key=|x| *x,
        )
      }
    }
    div(
      class="scroll-wrap",
      style="border: 2px solid black; width: 80%; height: 80%; max-width: 1400px; max-height: 1400px; margin: 5px"
    ) {
      // scrollbar
      div(ref=track_ref, class=if lateral {"scroll-track-x"} else {"scroll-track-y"}) {
        div(ref=thumb_ref, class=if lateral {"scroll-thumb-x"} else {"scroll-thumb-y"}, style="background-color: orange;") {}
      }
      // scrolling
      div(
        ref=wrap_ref, class="full no-scrollbar scrolling"
      ) {
        // passive
        Keyed(
          iterable=*cartons,
          view=move |x| view! {
            div(data-passive=x, class=if lateral {"carton carton-passive-x"} else {"carton carton-passive-y"}, style=format!("background-color: {};", d2rgb(x+100)).as_str()) {
              (x)
            }
          },
          key=|x| *x
        )
        // active
        Keyed(
          iterable=*cartons,
          view=move |x| view! {
            Carton(lateral=lateral, complex=complex, wrap_ref=wrap_ref, carton=x, terminal_carton=terminal_carton, update_scroll=update_scroll)
          },
          key=|x| *x
        )
      }
    }
    br()
    br()
  }
}


#[component(inline_props)]
pub fn Carton<G: Html>(
  lateral: bool, 
  complex: Signal<CartonsComplex<usize>>, 
  wrap_ref: NodeRef<G>, 
  carton: usize, 
  terminal_carton: ReadSignal<Option<usize>>,
  update_scroll: Signal<bool>
) -> View<G> {

  // resizer
  let (resizer_ref, resizing) = CartonsComplex::init_resizer(complex, wrap_ref, None, carton, None);

  create_effect(on(resizing, move || {
    alter_class(wrap_ref, "select-none", resizing.get());
    alter_class(resizer_ref, "moving", resizing.get());

    update_scroll.set(true);
  }));

  let resizer_ = create_memo(move || {
    view! { div(ref=resizer_ref, class=if lateral {"resizer resizer-x"} else {"resizer resizer-y"}) }
  });

  view! {
    div(data-carton=carton, class=if lateral {"carton carton-active-x"} else {"carton carton-active-y"}, style=format!("background-color: {};", d2rgb(carton+101)).as_str()) {
      (carton)
      (if terminal_carton.get()==Some(carton) { view! {} } else {
        view! { (resizer_.get_clone()) }
      })
    }
  }
}