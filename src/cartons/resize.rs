//! Resizing functions of [CartonsComplex]
//! 
//! This is for each "resizer", an actual resizing handler attached to each cartons.
//! By moving resizer with pointer(mouse cursor), we can resize cartons!
//! 
//! Resizing logic here might look complex, but it works sleek!


use super::*;
use std::cmp::Ordering;

impl<T: Eq + Hash + FromStr + Clone + std::fmt::Debug> CartonsComplex<T> {

  /// return Some(new_size)
  fn handle_expanding(
    &self,
    data: &T,
    delta: f64,
    wrap_size: f64,
    data_sizes: &mut Vec<(T, Option<f64>)>,
    index: usize,
    size: Option<f64>,
    min: f64,
    zero_restored: &mut HashSet<T>
  ) -> Option<f64> {

    if let Some(size) = size {
      // 1) not zeroed & larger/equal than min => have limitation check to change
      let new_size = size+delta;
      if new_size>=min {
        let new_size = self.max_limited(data, new_size, wrap_size);
        if size<new_size {
          data_sizes[index].1.replace(new_size);
          return Some(new_size);
        }
      }
    } else {
      // 2) was zeroed? => expand when raw new size is over zeroed_when
      if let Some(zeroed_when) = self._zeroed_when(data, 0.) {
        if delta>=zeroed_when {
          data_sizes[index].1.replace(min);
          zero_restored.remove(data);
          return Some(min);
        }
      }
    }

    None
  }

  /// return Some(new_size)
  fn handle_shrinking(
    &self,
    data: &T,
    delta: f64,
    wrap_size: f64,
    data_sizes: &mut Vec<(T, Option<f64>)>,
    index: usize,
    size: Option<f64>,
    min: f64,
    zeroed_cache: &mut HashMap<T, f64>,
  ) -> Option<f64> {

    // not yet zeroed
    if let Some(size) = size {
      // raw new size
      let new_size = size+delta;

      // 1) less than min => zero when delta.abs()(=size-new_size) is over zeroed_when
      if new_size<min {
        if let Some(zeroed_when) = self._zeroed_when(data, size) {
          if delta.abs()>zeroed_when {
            data_sizes[index].1.take();
            return Some(0.);
          }
        }
      // 2) larger/equal than min => shrink
      } else {
        data_sizes[index].1.replace(new_size);
        zeroed_cache.insert(data.clone(), size/wrap_size);
        return Some(new_size);
      }
    }

    None
  }

  /// Resizing just one carton.
  /// Expanding:
  ///   * zeroed and before zeroed_when: nothing
  ///   * zeroed and over zeroed_when: restore to min
  ///   * or: expand with limitation check
  /// 
  /// Shrinking:
  ///   * min limited & zeroable and over zeroed_when: zero shrinking
  ///   * min limited & zeroable and not yet over zeroed_when: do nothing
  ///   * or: shrink with limitation check
  fn independent_resizing(
    &self,
    data: &T,
    delta: f64,
    wrap_size: f64,
    data_sizes: &mut Vec<(T, Option<f64>)>,
    index: usize,
    size: Option<f64>,
    zeroed_cache: &mut HashMap<T, f64>,
    zero_restored: &mut HashSet<T>
  ) -> Result<()> {

    let Some(cmp) = delta.partial_cmp(&0.) else { return Err(Error::Ignore) };

    match cmp {
      Ordering::Equal => return Err(Error::Ignore),

      // expanding
      Ordering::Greater => {
        let min = self._min(data, wrap_size);
        if self.handle_expanding(data, delta, wrap_size, data_sizes, index, size, min, zero_restored).is_none() {
          return Err(Error::Ignore);
        }
      },
      Ordering::Less => {
        let min = self._min(data, wrap_size);
        if self.handle_shrinking(data, delta, wrap_size, data_sizes, index, size, min, zeroed_cache).is_none() {
          return Err(Error::Ignore);
        }
      }
    }

    Ok(())
  }

  /// Dependent resizing
  /// 
  /// 0. check `max shrink cap` (without zero-able case)
  /// 1. If nothing can be shrinked (`max shrink cap` is 0.):
  ///     * Try first shrink carton's zero-shrink, if available.
  ///     * Then sync it with exapanding' sizes.
  /// 2. Or, try first expand carton's zero-restore, if it's at zeroed state.
  ///     * If it's zeroed and not yet restore-able, do nothing (return err).
  /// 3. Or, do common expand and shrink, not caring about zeroed case.
  ///     * 1) expand caches
  ///     * 2) expand cartons from front ones
  ///     * 3) shrink from front ones
  fn dependent_resizing(
    &self,
    delta: f64,
    cache: *mut Vec<(usize, f64)>,
    wrap_size: f64,
    data_sizes: &mut Vec<(T, Option<f64>)>,
    index: usize,
    zeroed_cache: &mut HashMap<T, f64>,
    zero_restored: &mut HashSet<T>
  ) -> Result<()> {

    // a closure to add former-shrinking size to cache
    let add_to_cache = |cache: *mut Vec<(usize, f64)>, i: usize, size: f64| {
      unsafe {
        if (*cache).iter().all(|(i_, _)| *i_!=i) {
          (*cache).push((i, size));
        }
      }
    };

    let Some(cmp) = delta.partial_cmp(&0.) else { return Err(Error::Ignore) };

    let (raw_capacity, lead_index, rev) = match cmp {
      Ordering::Equal => return Err(Error::Ignore),
      Ordering::Greater => {
        (delta, index, false)
      },
      Ordering::Less => {
        (-delta, index+1, true)
      }
    };

    // get shrinks/expands, splitted at the boundary of lead_index, and sorted in order of closer to the lead_index
    let (shrinks, mut expands): (Vec<_>, Vec<_>) = {

      let list: Vec<(usize, T, Option<f64>, f64)> = data_sizes.iter().enumerate().map(|(i, (data, size))| {
        let min = self._min(data, wrap_size);
        (i, data.clone(), *size, min)
      }).collect();

      if rev {
        let mut shrinks = list;
        let expands = shrinks.split_off(lead_index);
        shrinks.reverse();
        (shrinks, expands)
      } else {
        let mut expands = list;
        let shrinks = expands.split_off(lead_index+1);
        expands.reverse();
        (shrinks, expands)
      }
    };

    // a closure to do shrink(no zeroing) within given capacity
    let do_shrink = 
      move |mut capacity: f64, shrinks: &Vec<(usize, T, Option<f64>, f64)>, data_sizes: &mut Vec<(T, Option<f64>)>, zeroed_cache: &mut HashMap<T, f64>| {

      for (i, data, size, min) in shrinks.iter() {
        if capacity<=0. {
          break;
        }
        if let Some(size) = size {
          let delta = -((*size-*min).min(capacity));
          if delta<0. {
            if let Some(new_size) = self.handle_shrinking(&data, delta, wrap_size, data_sizes, *i, Some(*size), *min, zeroed_cache) {
              let cap = *size-new_size;
              capacity -= cap;
              add_to_cache(cache, *i, *size);
            }
          }
        }
      }
    };

    // 0. check max shrink cap
    let max_shrink_cap = shrinks.iter().fold(0., |acc, (.., size, min) | {
      if let Some(size) = size { acc+(*size-*min).max(0.) } else { acc }
    });

    // 1. If nothing can be shrinked, try first's zero shrink
    if max_shrink_cap==0. {
      if let Some((i, data, size, _min)) = shrinks.first() {
        if let Some(size) = size {
          if let Some(zeroed_when) = self._zeroed_when(data, *size) {
            if raw_capacity>=zeroed_when {
              let capacity = *size;
  
              // check if expand capacity includes zero-shrink capacity
              let mut cap_ = capacity;
              let mut exps = vec![];
              for (i, data, size, _min) in expands.iter() {
                if cap_ <= 0. {
                  break;
                }
                if let Some(size) = size {
                  let max = self._max(data, wrap_size);
                  let cap = max.map(|max| (max-size).max(0.).min(cap_)).unwrap_or(cap_);
                  cap_ -= cap;
                  exps.push((*i, cap));
                }
              }
  
              if !exps.is_empty() && cap_==0. {
                // zero-shrink
                data_sizes[*i].1.take();
                zeroed_cache.insert(data.clone(), *size/wrap_size);
                add_to_cache(cache, *i, *size);
                // expand
                for (i, cap) in exps.into_iter() {
                  data_sizes[i].1.as_mut().map(|x| { *x+=cap; });
                }
                return Ok(());
              }
            }
          }
        }
      }
      return Err(Error::Ignore);
    }


    // 2. Try first's zero-restore
    let mut wait_zero_restore = false;

    if let Some((i, data, size, min)) = expands.first() {
      if size.is_none() {
        if let Some(zeroed_when) = self._zeroed_when(data, 0.) {
          if raw_capacity>=zeroed_when && *min<=max_shrink_cap {
            // restore
            data_sizes[*i].1.replace(*min);
            zero_restored.insert(data.clone());
            // shrink
            do_shrink(*min, &shrinks, data_sizes, zeroed_cache);
            return Ok(());
          }
          wait_zero_restore = true;
        }
      }
    }

    if wait_zero_restore {
      return Err(Error::Ignore)
    }


    // 3. Do expand and shrink
    let mut exp_capacity = raw_capacity.min(max_shrink_cap);
    let exp_capacity0 = exp_capacity;

    // 1) expand cap of cache
    let is_cache_at_expandings = |i: usize| {
      if rev { i>=lead_index } else { i<=lead_index }
    };

    unsafe {
      for (i, cache) in (*cache).iter().rev() {
        if exp_capacity<=0. {
          break;
        }
        if is_cache_at_expandings(*i) {
          if let Some((i, data, size, min)) = expands.iter_mut().find(|(i_, ..)| *i_==*i) {
            if let Some(size) = size {
              let delta = (*cache-*size).min(exp_capacity);
              if delta>0. {
                if let Some(new_size) = self.handle_expanding(&data, delta, wrap_size, data_sizes, *i, Some(*size), *min, zero_restored) {
                  let cap = new_size-*size;
                  *size = new_size;
                  exp_capacity -= cap;
                } 
              }
            }
          }
        }
      }
    }

    // 2) expand of cap from front
    for (i, data, size, min) in expands.iter_mut() {
      if exp_capacity<=0. {
        break;
      }
      if let Some(size) = size {
        if let Some(new_size) = self.handle_expanding(data, exp_capacity, wrap_size, data_sizes, *i, Some(*size), *min, zero_restored) {
          let cap = new_size-*size;
          *size = new_size;
          exp_capacity -= cap;
        }
      }
    }

    // 3) shrink (till min)
    let shrink_capacity = exp_capacity0-exp_capacity;

    do_shrink(shrink_capacity, &shrinks, data_sizes, zeroed_cache);

    Ok(())
  }
  
  /// Update sizing state,
  /// from a resizer with given `data` ...
  fn update_resize<E: AsRef<Element>>(
    &self,
    wrap: E,
    data: &T,
    delta: f64,
    cache: *mut Vec<(usize, f64)>,
    zeroed_cache: &mut HashMap<T, f64>,
    zero_restored: &mut HashSet<T>
  ) -> Result<CartonsMetric<T>> {

    let (wrap_size, elems, mut data_sizes, index, size) = self.measures(wrap, data)?;

    let (update_since, total_size) = 
      // independent case
      if self.independent {

        let _ = self.independent_resizing(data, delta, wrap_size, &mut data_sizes, index, size, zeroed_cache, zero_restored)?;
        let total_size = Self::get_total_size(&data_sizes);
        (index, total_size)

      // dependent case
      } else {
        let _ = self.dependent_resizing( delta, cache, wrap_size, &mut data_sizes, index, zeroed_cache, zero_restored)?;
        let total_size = self.adjust_to_fill_blank(wrap_size, &mut data_sizes);
        (0, total_size)
      };

    // ajust and upate style
    // return new metric
    
    let _ = self.update_style(elems, &data_sizes, update_since);
    let metric = CartonsMetric::new_from(data_sizes, total_size);

    Ok(metric)
  }


  /// Switch on/off zero state
  pub fn switch_zero<E: AsRef<Element>>(&mut self, wrap: E, data: &T, on: bool) -> Result<()> {

    let (wrap_size, elems, mut data_sizes, index, size) = self.measures(wrap, data)?;

    let mut changed = false;

    if on {
      if size.is_none() {
        let cache = self._zeroed_cache(data, wrap_size);
        let min = self._min(data, wrap_size);

        if self.independent {
          if cache>=min {
            data_sizes[index].1.replace(cache);
            self.zeroed_cache.remove(data);
            changed = true;
          }
        } else{
          let mut cap_ = cache;
          let mut xx = vec![];
          for (i, (data, size)) in data_sizes.iter().enumerate() {
            if cap_<=0. { break; }
            if let Some(size) = size {
              let min = self._min(data, wrap_size);
              let cap = (*size-min).max(0.).min(cap_);
              if cap>0. {
                cap_ -= cap;
                xx.push((i, cap));
              }
            }
          }
          let cap = cache.min(cache-cap_);
          if cap>=min {
            data_sizes[index].1.replace(cap);
            self.zeroed_cache.remove(data);
            for (i, cap) in xx {
              data_sizes[i].1.as_mut().map(|x| { *x-=cap; });
            }
            changed = true;
          }
        }
      }
    } else if let Some(size) = size {
      if *self.allow_zero.get(data) {
        if self.independent {
          data_sizes[index].1.take();
          self.zeroed_cache.insert(data.clone(), size/wrap_size);
          changed = true;
        } else {
          let mut cap_ = size;
          let mut xx = vec![];
          for (i, (data, size)) in data_sizes.iter().enumerate() {
            if cap_<=0. { break; }
            if let Some(size) = size {
              let max = self._max(data, wrap_size);
              let cap = max.map(|max| (max-size).max(0.).min(cap_)).unwrap_or(cap_);
              cap_ -= cap;
              xx.push((i, cap));
            }
          }
          if cap_==0. {
            data_sizes[index].1.take();
            self.zeroed_cache.insert(data.clone(), size/wrap_size);
            for (i, cap) in xx {
              data_sizes[i].1.as_mut().map(|x| { *x+=cap; });
            }
            changed = true;
          }
        } 
      }
    }


    if !changed {
      return Err(Error::Ignore)
    }

    // ajust and upate style
    // return new metric
    let total_size = {
      if self.independent {
        Self::get_total_size(&data_sizes)
      } else {
        self.adjust_to_fill_blank(wrap_size, &mut data_sizes)
      }
    };

    let _ = self.update_style(elems, &data_sizes, 0);
    let metric = CartonsMetric::new_from(data_sizes, total_size);

    self.metric = metric;

    Ok(())
  }

  /// Conduct resizing job of a resizer, which is attached to each carton and manually resizes with pointerdown/move event.
  pub fn resize_work<X: Copy + 'static, E: AsRef<Element>>(
    &self,
    e: PointerEvent,
    data: &T,
    pos: *mut Option<f64>,
    shift: *mut Option<f64>,
    cache: *mut Vec<(usize, f64)>,
    wrap: X,
    resizer: X,
    get_elem: impl Fn(X) -> Option<E> + Copy + 'static,
  ) -> Result<(CartonsMetric<T>, HashMap<T, f64>, HashSet<T>)> {

    let Some(wrap) = get_elem(wrap) else { return Err(Error::Ignore) };
    let Some(resizer) = get_elem(resizer) else { return Err(Error::Ignore) };
    let Some(shift) = (unsafe { *shift }) else { return Err(Error::Ignore) };

    let front = get_elem_front(resizer, self.lateral) - shift;
    let client_pos = if self.lateral { e.client_x() as f64 } else { e.client_y() as f64 };
    let delta = client_pos - front;

    // be careful about the unsafe scope! 
    let Some(pos0) = (unsafe {(*pos).replace(client_pos) }) else { return Err(Error::Ignore) };
    let moving = client_pos - pos0;

    if moving==0. || delta==0. {
      return Err(Error::Ignore)
    }
    if (moving>0.) != (delta>0.) {
      return Err(Error::Ignore);
    }

    let mut zeroed_cache = HashMap::new();
    let mut zero_restored = HashSet::new();

    let metric = self.update_resize(wrap, data, delta, cache, &mut zeroed_cache, &mut zero_restored)?;
    Ok((metric, zeroed_cache, zero_restored))
  }

  /// Expand [`resize_work()`] for Sycamore.
  /// Initialte a resizer handler, which is attached to each carton and manually resizes with pointerdown/move event.
  /// 
  /// *feature `sycamore`*
  #[cfg(feature="sycamore")]
  pub fn init_resizer<G: GenericNode>(
    complex: Signal<Self>,
    wrap_ref: NodeRef<G>,
    resizer_ref: Option<NodeRef<G>>,
    data: T,
    resizing: Option<Signal<bool>>
  ) -> (
    NodeRef<G>,
    Signal<bool>
  ) {

    let resizer_ref = resizer_ref.unwrap_or(create_node_ref());
    let resizing = resizing.unwrap_or(create_signal(false));

    let pos: *mut Option<f64> = Box::into_raw(Box::new(None));
    let shift: *mut Option<f64> = Box::into_raw(Box::new(None));
    let cache: *mut Vec<(usize, f64)> = Box::into_raw(Box::new(vec![]));

    let pointer_move = move |e: PointerEvent| {

      if let Ok((metric, zeroed_cache, zero_restored)) = 
        complex.with(|complex| complex.resize_work(e, &data, pos, shift, cache, wrap_ref, resizer_ref, ref_get::<_, Element>))
      {  
        complex.update(|complex| {
          complex.metric = metric;
          for x in zero_restored.iter() {
            complex.zeroed_cache.remove(x);
          }
          for (k, v) in zeroed_cache.into_iter() {
            complex.zeroed_cache.insert(k, v);
          }
        });
      }
    };

    let pointer_up = move |_| {
      unsafe {
        (*cache).clear();
        let _ = (*shift).take();
        let _ = (*pos).take();
      }
      resizing.set(false);
    };

    let pointer_down = move |e: PointerEvent| {
      complex.with(|complex| {
        unsafe {
          if let Some(resizer) = ref_get::<_, Element>(resizer_ref) {
            let front = get_elem_front(resizer, complex.lateral);
            let client_pos = if complex.lateral { e.client_x() as f64 } else { e.client_y() as f64 };
            let _ = (*shift).replace(client_pos - front);
            let _ = (*pos).replace(client_pos);
          }
        }
      });
      resizing.set(true);
    };

    let (cb_pointerdown, raws) = pointer_down_move_up(pointer_down, pointer_move, pointer_up);


    on_mount(move || {
      ref_get::<_, EventTarget>(resizer_ref).map(|resizer| {
        resizer.add_event_listener_with_callback("pointerdown", cb_pointerdown.as_ref().unchecked_ref()).unwrap_throw();
        
        on_cleanup(move || {
          resizer.remove_event_listener_with_callback("pointerdown", cb_pointerdown.as_ref().unchecked_ref()).unwrap_throw();
        });
      });

      on_cleanup(move || {
        (raws, cache, shift).clean();
      });
    });

    (resizer_ref, resizing)
  }
}