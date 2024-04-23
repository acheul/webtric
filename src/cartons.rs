//! Many modern web apps support multi tap. And the multip tab, or paralle panels, are supposed to be *resized*.
//! 
//! For resizing, we need to
//! * track tab/panels' currnet size
//! * have resizing handler/bar which resizes on its pointerdown/move events.
//! * also update total sizes with window reisize event or any other events can effect sizing states.
//! 
//! Here, we will call parallel tab/panels as "cartons".
//! * [`CartonsComplex`] captures (re)sizing rules and tracks current size state of a complex of cartons.
//!   `CartonsComplex` has associated functions to handle sizing/resizing problems.
//! * Each carton under a complex is required to have unique [`dataset`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dataset) 
//!   value matched with a unified dataset name(key).
//! * [`CartonsMap`] is a helper data structure to store information of sizing rules and states.
//! * [`CartonsMetric`] is deviated from `CartonsMap`.

use crate::*;

pub mod resize;


/// Helper structure to store data of sizing rules and states.
/// Generic `<T>` is a carton's dataset value type. Check [`parse_dataset()`] about this.
/// * Field `map` stores specific carton's data.
/// * If a carton's name is not in the `map`, get field `default`'s value as fallback.
#[derive(Default, Debug, Clone)]
pub struct CartonsMap<T: Eq + Hash + FromStr + Clone, V> {
  pub map: HashMap<T, V>,
  pub default: V
}

impl<T: Eq + Hash + FromStr + Clone, V: Default> Into<CartonsMap<T, V>> for Vec<(T, V)> {
  /// Build a [CartonsMap] from Vec<(T, V)>.
  /// `default` field will be default value of `<V>`.
  fn into(self) -> CartonsMap<T, V> {
    CartonsMap {
      map: self.into_iter().collect(),
      default: V::default()
    }
  }
}

impl<T: Eq + Hash + FromStr + Clone, V> Into<CartonsMap<T, V>> for (Vec<(T, V)>, V) {
  /// Build a [CartonsMap] from (Vec<T, V>, V).
  /// First element of tuple will be `map` field, and second one goes to `default` field.
  fn into(self) -> CartonsMap<T, V> {
    CartonsMap {
      map: self.0.into_iter().collect(),
      default: self.1
    }
  }
}


impl<T: Eq + Hash + FromStr + Clone, V> CartonsMap<T, V> {
  /// Make new CartonsMap. Using `Into` trait might be simpler than this.
  pub fn new(list: Vec<(T, V)>, default: V) -> Self {

    Self { map: list.into_iter().collect(), default }
  }

  /// Getting a stored information with given dataset name, `data`.
  /// If `data` is in the `map` of struct, return it's corresponding value. Or return `default`.
  /// 
  /// # Example
  /// ```
  /// # use webtric::CartonsMap;
  /// let map: CartonsMap<usize, f64> = vec![(0, 10.), (1, 20.)].into();
  /// let map2: CartonsMap<usize, f64> = (vec![(0, 10.)], 20.).into();
  /// 
  /// assert_eq!(*map.get(&0), 10.);
  /// assert_eq!(*map2.get(&0), 10.);
  /// assert_eq!(*map.get(&1), 20.);
  /// assert_eq!(*map2.get(&1), 20.);
  /// assert_eq!(*map.get(&2), 0.);
  /// assert_eq!(*map2.get(&2), 20.);
  /// ```
  pub fn get(&self, data: &T) -> &V {
    if let Some(x) = self.map.get(data) {
      x
    } else {
      &self.default
    }
  }

  pub fn remove(&mut self, data: &T) -> () {
    self.map.remove(data);
  }

  pub fn insert(&mut self, data: T, value: V) -> () {
    self.map.insert(data, value);
  }
}


/// Alias of `CartonsMap<T, Option<Sizon>>`
/// 
/// value None refers to "zeroed" state;
pub type CartonsMetric<T> = CartonsMap<T, Option<Sizon>>;

impl<T: Eq + Hash + FromStr + Clone> CartonsMetric<T> {

  /// Return new CartonsMetric, cloning self but revises only Sizon's `abs` field, keeps `rel`.
  /// It's a helper for CartonsComplex's `wrap_effect_on_update`
  fn abs_revised(
    &self,
    data_sizes: Vec<(T, Option<f64>)>,
    total_size: f64,
  ) -> Self {

    let map: HashMap<T, Option<Sizon>> = data_sizes.into_iter().map(|(data, size)| {
      if let Some(size) = size {
        let mut sizon = *self.get(&data);
        if let Some(sizon) = sizon.as_mut() {
          sizon.abs = Some(size);
        }
        (data, sizon)
      } else {
        (data, None)
      }
    }).collect();

    Self { map, default: Some(Sizon::abs(total_size)) }
  }

  /// Build new CartonsMetric from given args.
  /// It's a helper for `CartonsComplex's `update_resize`.
  fn new_from(
    data_sizes: Vec<(T, Option<f64>)>,
    total_size: f64
  ) -> Self {
    let map: HashMap<T, Option<Sizon>> = data_sizes.into_iter().map(|(data, size)| {
      let sizon = size.map(|size| Sizon::new(Some(size), Some(size/total_size)));
      (data, sizon)
    }).collect();
    
    Self { map, default: Some(Sizon::abs(total_size)) }
  }
}


/// CartonsComples captures (re)sizing rules and tracks current sizing state of a complex of cartons.
/// * (re)sizing rules of complex's wrapping level.
/// * resizer handler. => module `resize`
///
/// Each carton's supposed to have unique html dataset value matched with dataset name of field `name`.
#[derive(Debug, Clone)]
pub struct CartonsComplex<T: Eq + Hash + FromStr + Clone> {
  /// cartons arrangement direction (lateral or vertical?)
  pub lateral: bool,
  /// resizing independently or relatively
  /// * not independent setting will force the complex fill blank space.
  pub independent: bool,
  /// dataset name
  pub name: &'static str,
  /// metric: track current sizing state;
  /// * value None refers to "zeroed" state. (See [`CartonsMetric`])
  /// * The `default` field of it will refer to total size.
  pub metric: CartonsMetric<T>,
  /// min map: minimum size limit
  pub min: CartonsMap<T, Sizon>,
  /// max map: maximum size limit
  pub max: CartonsMap<T, Sizon>,
  /// allow zero
  /// * If "zero" is allowed, a carton can be shrinked into zero size, even though it's minimum limit is larger than zero.
  pub allow_zero: CartonsMap<T, bool>,
  /// zeroed when
  /// * This value decides at what "threshold" a carton can be zero-shrinked or restored-from zero size. 
  /// * if a carton is not `allow_zero`ed, this field will be ignored anyway.
  /// * Ex.
  ///   * Let's say a carton has minimum limit 30.(px) and zero allowed.
  ///     And its `zeroed_when` value is 10.(px). (Hereby not using sizon for simplication).
  ///     When a resizer's pointer(may be a mouse cursor) moves inwards passing minimum limit threshold, 30.px,
  ///     then moves more than 10.px, the carton will be zero-shrinked.
  ///     Later, when the pointer is moving outwards more than 10.px from the zero-shrinked carton,
  ///     then the carton's size will be restored to its minimum threshold, 30.px.
  pub zeroed_when: CartonsMap<T, Sizon>,
  /// cache former size's ratio
  pub zeroed_cache: CartonsMap<T, f64>
}


impl<T: Eq + Hash + FromStr + Clone + std::fmt::Debug> CartonsComplex<T> {

  /// New Cartons.
  /// If `name` is none, it will use "carton" for its `name` field.
  pub fn new(
    lateral: bool,
    independent: bool,
    name: Option<&'static str>,
    metric: CartonsMap<T, Option<Sizon>>,
    min: CartonsMap<T, Sizon>,
    max: CartonsMap<T, Sizon>,
    allow_zero: CartonsMap<T, bool>,
    zeroed_when: CartonsMap<T, Sizon>,
    zeroed_cache: CartonsMap<T, f64>
  ) -> Self {
    let name = name.unwrap_or("carton");
    Self { lateral, independent, name, metric, min, max, allow_zero, zeroed_when, zeroed_cache }
  }

  /// Make lists of carton element and dataset value, from given `wrap` element and dataset `name`
  fn wrap_to_carton_elems<E: AsRef<Element>>(wrap: E, name: &str) -> (Vec<HtmlElement>, Vec<T>) {

    let elems = wrap.as_ref().children();
    
    (0..elems.length()).filter_map(|i| elems.item(i).map(|carton| {
      let carton: HtmlElement = carton.unchecked_into();
      let data = parse_dataset::<_, T>(&carton, name);
      data.map(|data| (carton, data))
    }).flatten()).unzip()
  }

  /// return max limited size 
  fn max_limited(&self, data: &T, size: f64, wrap_size: f64) -> f64 {
    self.max.get(data).min(size, Some(wrap_size))
  }

  /// return min and max limited size
  fn limited(&self, data: &T, size: f64, wrap_size: f64) -> f64 {
    let size = self.max.get(data).min(size, Some(wrap_size));
    self.min.get(data).max(size, Some(wrap_size))
  }

  /// return possible max limitation size
  fn _max(&self, data: &T, wrap_size: f64) -> Option<f64> {
    let sizon = self.max.get(data);

    let _max = if let Some(abs) = sizon.abs {
      if let Some(rel) = sizon.rel {
        abs.min(rel*wrap_size) // use .min(), cuz it's for limitation check
      } else {
        abs
      }
    } else if let Some(rel) = sizon.rel {
      rel*wrap_size
    } else {
      return None;
    };
    Some(_max)
  }

  /// return possible min limitation size. fallback is zero.
  fn _min(&self, data: &T, wrap_size: f64) -> f64 {
    let sizon = self.min.get(data);

    if let Some(abs) = sizon.abs {
      if let Some(rel) = sizon.rel {
        abs.max(rel*wrap_size) // use .max(), as it's for limitation check
      } else {
        abs
      }
    } else if let Some(rel) = sizon.rel {
      rel*wrap_size
    } else {
      0.
    }
  }

  /// Return `zeroed_when` threshold value.
  /// Return None when it's not zeroed-able.
  fn _zeroed_when(&self, data: &T, size: f64) -> Option<f64> {

    if !*self.allow_zero.get(data) {
      return None;
    }

    let sizon = self.zeroed_when.get(data);

    let x = if let Some(abs) = sizon.abs {
      if let Some(rel) = sizon.rel {
        abs.max(rel*size)
      } else {
        abs
      }
    } else if let Some(rel) = sizon.rel {
      rel*size
    } else {
      0.
    };
    // Don't limit by size: size could be zero when it's zero shrinked.
    Some(x.max(0.))
  }

  /// Return if the carton is "zeroed".
  fn _zeroed(&self, data: &T) -> bool {
    // <!> Use below code to get expected result.
    // Using `self.metric.get()` or `self.metric.map.get()` may result into different output.
    self.metric.map.get(data).map(|x| x.is_none()).unwrap_or(false)
  }

  fn get_total_size(data_sizes: &Vec<(T, Option<f64>)>) -> f64 {
    data_sizes.iter().fold(0., |acc, (_, size)| {
      if let Some(size) = size {
        acc+*size
      } else {
        acc
      }
    })
  }

  fn _zeroed_cache(&self, data: &T, wrap_size: f64) -> f64 {
    let ratio = *self.zeroed_cache.get(data);
    wrap_size*ratio
  }


  /// Adjust data_sizes to fill blank space.
  /// Return adjusted total_size.
  /// 
  /// Not independent cartons will use it for wrapping level sizing and resizer's resizing.
  fn adjust_to_fill_blank(
    &self,
    wrap_size: f64,
    data_sizes: &mut Vec<(T, Option<f64>)>
  ) -> f64 {
    let mut total_size = Self::get_total_size(data_sizes);

    let mut blank = (wrap_size-total_size).floor();
    if blank>0. {
      // 1. proportional distribution
      for (data, size) in data_sizes.iter_mut() {
        if blank>0. {
          if let Some(size) = size {
            let add = *size/wrap_size * blank;
            let new_size = self.max_limited(data, *size+add, wrap_size);
            let delta = (new_size-*size).max(0.);
            total_size += delta;
            blank -= delta;
            *size += delta;
          }
        } else {
          break
        }
      }

      // 2. distribute from rear ones
      if blank>0. {
        for (data, size) in data_sizes.iter_mut().rev() {
          if blank>0. {
            if let Some(size) = size {
              let new_size = self.max_limited(data, *size+blank, wrap_size);
              let delta = (new_size-*size).max(0.);
              total_size += delta;
              blank -= delta;
              *size += delta;
            } 
          } else {
            break;
          }
        }
      }
    }

    total_size
  }

  /// Udpate each cartons front position and size style.
  /// Conduct update from `since` index.
  /// 
  /// Memo: We'are not using right2left or bottom2top complex, as they are highly likely to unsync with browser's work.
  fn update_style<H: AsRef<HtmlElement>>(
    &self,
    elems: Vec<H>,
    data_sizes: &Vec<(T, Option<f64>)>,
    since: usize
  ) {
    let (size_prop, pos_prop) = size_pos_props(self.lateral);
    let mut pos = 0.;
    elems.into_iter().zip(data_sizes.iter()).enumerate().for_each(|(i, (elem, (.., size)))| {
      let size = size.unwrap_or(0.);
      if i>=since {
        let style = elem.as_ref().style();
        style.set_property(size_prop, format!("{:.2}px", size).as_str()).unwrap_throw();
        style.set_property(pos_prop, format!("{:.2}px", pos).as_str()).unwrap_throw();
      }
      pos += size;
    });
  }

  /// Wrapping level's effect on any possible update (such as initiation and window's resizing).
  /// 
  /// Get new metric data at the moment, update sizing style, then return the metric data.
  /// It's updating carton sizes from former metric data's relative size information. (`rel` of [Sizon]).
  /// Thus, updates like window resizing would be able to preserve overall relative size scheme.
  /// 
  /// It's generalized function. More applicated ones:
  /// * *sycamore* => [`init_wrap()`]
  /// * ~~*leptos* => [`leptos_init_wrap()`]~~
  pub fn wrap_effect_on_update<X: Copy + 'static, E: AsRef<Element>>(
    &self,
    wrap: X,
    get_elem: impl Fn(X) -> Option<E> + Copy + 'static,
  ) -> Result<CartonsMetric<T>> {

    let Some(wrap) = get_elem(wrap) else { return Err(Error::Ignore) };
    let wrap_size = get_client_size(&wrap, self.lateral);

    let (elems, datas) = Self::wrap_to_carton_elems(&wrap, self.name);

    // use sizon.rel and limitation
    let mut data_sizes: Vec<(T, Option<f64>)> = datas.into_iter().map(|data| {
      let sizon = self.metric.get(&data);
      let size = sizon.map(|sizon| {
        let size = if let Some(rel) = sizon.rel {
          wrap_size*rel
        } else {
          sizon.abs.unwrap_or_default()
        };
        self.limited(&data, size, wrap_size)
      });
      
      (data, size)
    }).collect();

    let total_size = if self.independent {
      Self::get_total_size(&data_sizes)
    } else {
      self.adjust_to_fill_blank(wrap_size, &mut data_sizes)
    };

    self.update_style(elems, &data_sizes, 0);

    let metric = self.metric.abs_revised(data_sizes, total_size);
    Ok(metric)
  }

  /// Exapnd [`wrap_effect_on_update()`] for ready made use in Sycamore
  /// 
  /// * This makes a `create_effect` listening to `update_by`,
  ///   which then update total sizing state and signal `complex`'s metric data.
  /// * The `udpate_by` signals are supposed to be triggered on initiation so as to initiate the sizing state.
  /// 
  /// # Args
  /// * complex: the signal of CartonsComplex
  /// * wrap_ref: wrapping element's NodeRef
  /// * update_by: tuple of signals which can effect sizing states(implementing sycamore's `Trackable` trait).
  ///   Ex. window_resizing signal
  /// 
  /// # Outputs
  /// * wrap_ref
  #[cfg(feature="sycamore")]
  pub fn init_wrap<G: GenericNode, U: Trackable + 'static>(
    complex: Signal<Self>,
    wrap_ref: Option<NodeRef<G>>,
    update_by: U
  ) -> NodeRef<G> 
  {
    let wrap_ref = wrap_ref.unwrap_or(create_node_ref());

    on_mount(move || {
      create_effect(on(update_by, move || {

        if let Ok(metric) = 
          complex.with_untracked(|complex| // MUST use untracked
            complex.wrap_effect_on_update(wrap_ref, ref_get::<_, Element>)
          ) 
        {
          complex.update(|complex| {
            complex.metric = metric;
          });
        }
      }));
    });

    wrap_ref
  }

  /// Passive wrap's effect on any update of cartons.
  /// Update sizing style using given complex's metric's `abs` info.
  /// 
  /// It's generalized function. More applicated ones:
  /// * *sycamore* => [`init_passive_wrap()`]
  pub fn passive_wrap_effect_on_update<X: Copy + 'static, E: AsRef<Element>>(
    &self,
    wrap: X,
    get_elem: impl Fn(X) -> Option<E> + Copy + 'static,
  ) -> Result<()> {

    let Some(wrap) = get_elem(wrap) else { return Err(Error::Ignore) };

    // sizes_datas and update style
    let (cartons, datas) = Self::wrap_to_carton_elems(&wrap, self.name);

    // gather using sizon.abs
    let data_sizes: Vec<(T, Option<f64>)> = datas.into_iter().map(|data| {
      let size = self.metric.get(&data).map(|x| x.abs).unwrap_or_default();
      (data, size)
    }).collect();

    self.update_style(cartons, &data_sizes, 0);
    
    Ok(())
  }

  /// Expand [`passive_wrap_effect_on_update`] for Sycamore.
  /// Initiate a passive wrapping complex, which would update sizing passively by signal `complex`'s metric data.
  #[cfg(feature="sycamore")]
  pub fn init_passive_wrap<G: GenericNode>(
    complex: Signal<Self>,
    wrap_ref: Option<NodeRef<G>>,
  ) -> NodeRef<G> {
    let wrap_ref = wrap_ref.unwrap_or(create_node_ref());

    on_mount(move || {
      create_effect(on(complex, move || {
        let _ = complex.with(|complex| complex.passive_wrap_effect_on_update(wrap_ref, ref_get::<_, Element>));
      }));
    });

    wrap_ref
  }

  /// Measures wrap element and its cartons' sizes.
  /// return (wrap-size, carton-elems, data-sizes, index, size)
  /// 
  /// This function checks if a carton is at "zeroed" state or not and reflect it to its output.
  fn measures<E: AsRef<Element>>(
    &self, 
    wrap: E,
    data: &T
  ) -> Result<(f64, Vec<HtmlElement>, Vec<(T, Option<f64>)>, usize, Option<f64>)> {

    let wrap_size = get_client_size(wrap.as_ref(), self.lateral);
    
    let elems = wrap.as_ref().children();
    
    let (elems, data_sizes): (Vec<_>, Vec<_>) = 
      (0..elems.length()).filter_map(|i| elems.item(i).map(|carton| {
        let carton: HtmlElement = carton.unchecked_into();
        if let Some(data) = parse_dataset::<_, T>(&carton, self.name) {
          let size = if self._zeroed(&data) {
            None
          } else {
            Some(get_elem_size(&carton, self.lateral)) // DO NOT use client size. USE DomRect size.
          };
          Some((carton, (data, size)))
        } else {
          None
        }    
      }).flatten()).unzip();

    let Some((index, size)) = data_sizes.iter().enumerate().find_map(|(i, (data_, size))| {
      if data_==data {
        Some((i, *size))
      } else {
        None
      }
    }) else { return Err(Error::Msg(String::from("data is not found"))) };

    Ok((wrap_size, elems, data_sizes, index, size))
  }
}