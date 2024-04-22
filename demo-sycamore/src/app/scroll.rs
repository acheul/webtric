use super::*;


#[component]
pub fn Scroll<G: Html>() -> View<G> {

  let lateral = create_signal(false);
  let take_ortho = create_signal(true);
  let update_by = create_signal(false);
  let cartons: Signal<Vec<usize>> = create_signal((0..12).collect());
  let scroll_metric_ = create_signal(ScrollMetric::default());

  let demo = create_memo(on((lateral, take_ortho), move || {
    view! {
      ScrollDemo(
        lateral=lateral.get(),
        take_ortho=take_ortho.get(),
        update_by=update_by,
        cartons=cartons,
        scroll_metric_=scroll_metric_
      )
    }
  }));

  view! {
    Index {
      h1(style="margin-left: 16px;") { "Test Scroll"}
      div(style="margin: 8px 16px;") {
        CheckBox(name="lateral", signal=lateral)
        CheckBox(name="take_orthogonal", signal=take_ortho)
      }
  
      // display metric
      div(style="margin: 8px 16px;") {
        div() {"metric(x):"}
        div(style="margin-left: 8px;") {
          div() { span() { "client_size: "} span() {(scroll_metric_.with(|x| x.x.client_size))}}
          div() { span() { "scroll_size: "} span() {(scroll_metric_.with(|x| x.x.scroll_size))}}
          div() { span() { "scroll_pos: "} span() {(scroll_metric_.with(|x| x.x.scroll_pos))}}
        }
        div() {"metric(y):"}
        div(style="margin-left: 8px;") {
          div() { span() { "client_size: "} span() {(scroll_metric_.with(|x| x.y.client_size))}}
          div() { span() { "scroll_size: "} span() {(scroll_metric_.with(|x| x.y.scroll_size))}}
          div() { span() { "scroll_pos: "} span() {(scroll_metric_.with(|x| x.y.scroll_pos))}}
        }
      }
  
      div(style="margin: 8px 6px;") {
        button(style="margin-right: 8px;", on:click=move |_| { push_carton(cartons, vec![update_by], None) }) {"+ push to last"}
        button(on:click=move |_| { pop_carton(cartons, vec![update_by]) }) {"- pop up first"}
      }
  
      (demo.get_clone())
    }
  }
}



#[component(inline_props)]
fn ScrollDemo<G: Html>(
  lateral: bool, take_ortho: bool,
  update_by: Signal<bool>,
  cartons: Signal<Vec<usize>>,
  scroll_metric_: Signal<ScrollMetric>
) -> View<G> {

  let WindowResizing(window_resizing) = use_context();

  let x_take_ortho = lateral && take_ortho;
  let y_take_ortho = !lateral && take_ortho;
  let x_bar = lateral;

  let (scrolling_ref, scroll_metric, scroll_x_to, _scroll_y_to, x, y, thumb_moving) =
    ScrollMetric::init_scrolling_and_scrollbars(x_take_ortho, y_take_ortho, (*window_resizing, *update_by), x_bar, !x_bar);
  
  let (track_ref, thumb_ref) = if lateral { x.unwrap() } else { y.unwrap() };

  on_mount(move || {
    create_effect(on(thumb_moving, move || {
      alter_class(scrolling_ref, "select-none", thumb_moving.get());
    }));
    create_effect(on(scroll_metric, move || {
      let b = scroll_metric.with(|metric| if lateral { metric.x.scrollable() } else { metric.y.scrollable() });
      alter_class(track_ref, "opacity0", !b);
      alter_class(thumb_ref, "opacity0", !b);
    }));
  });

  scroll_x_to.set(20.);

  create_effect(on(scroll_metric, move || {
    scroll_metric_.set(scroll_metric.get());
  }));

  view! {
    div(
      class="scroll-wrap",
      style="border: 2px solid black; width: 80%; height: 80%; max-width: 800px; max-height: 800px; margin: 5px"
    ) {
      div(ref=track_ref, class=if lateral {"scroll-track-x"} else {"scroll-track-y"}) {
        div(ref=thumb_ref, class=if lateral {"scroll-thumb-x"} else {"scroll-thumb-y"}, style="background-color: orange;") {}
      }
      // scrolling
      div(
        ref=scrolling_ref, class="full no-scrollbar scrolling"
      ) {
        Keyed(
          iterable=*cartons,
          view=move |x| view! {
            div(style="display: flex; flex-direction: row;") {
              Keyed(
                iterable=*cartons,
                view=move |y| view! {
                  div(
                    class="scroll-carton",
                    style=format!("background-color: {};", d2rgb(x+y)).as_str()
                  ) { (format!("({x},{y})")) }
                },
                key=|x| *x
              )
            }
          },
          key=|x| *x
        )
      }
    }
    br()
    br()
  }
}