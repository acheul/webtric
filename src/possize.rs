//! # Positioned Size: Decide position of tooltips/menubars
//! 
//! Tooltips or context menubars has undetermined positions until they are triggered to be shown up.
//! In most cases, tooltips have style of {position: absolute}, which is relative to its ancestor element,
//! while context menubars have style of {position: fixed}, and drawn at a coordinate where mouse click event happened.
//! 
//! For any cases, they need to be reactive to environment:
//! * A tooltip might be designed to be shown up at upper space of its ancestor element, however when
//!   there is not enought space at upper side at the moment of being drawn, it needs to adjust its position.
//! * A menubar might be intended to be drawn at right-down side of mouseclick position, however if the click happend
//!   at right-down corner of the browser, the menubar should move its poisition.
//! 
//! This module is to solve this problem: reactively decide position at the moment of drawing
//! considering
//!   * brower's client size,
//!   * ancestor's size and position, 
//!   * and pre-assinged configures like drawing tooltip/menubar's size, margin, and their relative position to its ancestor.
//! 
//! For tooltips with {position: absolute}, use `AbsPosSize`.
//! 
//! For context menubars with {position: fixed}, use `FixedPosSize`.

use crate::*;

/// trait for `AbsUniPosSize` and `FixedUniPosSize`
pub trait UniPosSize {

  /// Size of an element. (width or height)
  fn size(&self) -> f64;

  /// front side margin of an element.
  /// (hereby "front side" refers to `left` or `top`)
  /// 
  /// In metric sense, this margin also can be understood as buffer space of browser(document)'s client space.
  fn front_margin(&self) -> f64;

  /// rear side margin of an element.
  /// ("rear side" means `right` or `bottom`)
  /// 
  /// In metric sense, this margin also can be understood as buffer space of browser(document)'s client space.
  fn rear_margin(&self) -> f64;

  /// Does a relevant element's size go over browser's client range?
  /// 
  /// Calculation:
  ///   * the element's size range is [`fornt_fixed_pos`, `front_fixed_pos` + `self.size`]
  ///   * Check is this range located inside of the browser's client range [`self.front_margin`, document-size - `self.rear_margin`]
  ///   * (The `front_margin` and `rear_margin` can be handled in respect to either "element's margin" or "browser's buffer space".
  ///     In context of computation, either way doesnt' matter.)
  fn is_over(&self, front_fixed_pos: f64, doc_size: f64) -> bool {
    front_fixed_pos<self.front_margin() ||
    front_fixed_pos + self.size() > doc_size - self.rear_margin()
  }

  /// Adjust given `front_fixed_pos` not to go over browser's client range.
  /// 
  /// Not exceeding the front side has a higher priority than not excedding the rear side.
  fn adjust_front_pos(&self, front_fixed_pos: &mut f64, doc_size: f64) {
      
    let rear_pos = *front_fixed_pos + self.size();
    let over = rear_pos - (doc_size - self.rear_margin());
    if over > 0. {
      *front_fixed_pos -= over;
    }

    let over = self.front_margin() - *front_fixed_pos;
    if over > 0. {
      *front_fixed_pos += over;
    }
  }
}

/// # Fixed Positioned Size (Uni dimensional)
/// 
/// A relevant element is supposed to have style { position: fixed; }
#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FixedUniPosSize  {
  /// one's pre-assigned size
  pub size: f64,
  /// one's front side margin (left/top side)
  pub front_margin: f64,
  /// one's rear side margin (right/bottom side)
  pub rear_margin: f64
}

impl UniPosSize for FixedUniPosSize {
  fn size(&self) -> f64 { self.size }
  fn front_margin(&self) -> f64 { self.front_margin }
  fn rear_margin(&self) -> f64 { self.rear_margin }
}

impl FixedUniPosSize {
  pub fn new(size: f64, front_margin: f64, rear_margin: f64) -> Self {
    Self {
      size, front_margin, rear_margin
    }
  }

  /// Return adjusted front_fixed_pos(`left` or `top`) of an element, considering given position and document size
  pub fn front_fixed_pos(&self, mut client_pos: f64, doc_size: f64) -> f64 {
    
    if self.is_over(client_pos, doc_size) {
      self.adjust_front_pos(&mut client_pos, doc_size);
    }
    client_pos
  }
}


/// # Absolute Positioned Size (Uni dimensional)
/// 
/// A relevant element is supposed to have style { position: absolute; }.
/// Thus it's positioned relatively to its relevant ancestor.
/// 
/// Configure the element's relative position to its ancestor:
///   * front: is its position relative to ancestor' left/top or right/bottom?
///   * outward: is the element spreading outwards or inwards in respect to its ancestor?
///   * gap: gap between oneself and ancestor's front/rear. Use [`Sizon`].
///     - When trying to get a value from sizon, the priority is: 1) field `abs` 2) `rel` 3) fallback return 0.
#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AbsUniPosSize  {
  /// from ancestor's front/rear
  pub front: bool,
  /// spreading outwards or inwards in respect to ancestor
  pub outward: bool,
  /// gap between oneself and ancestor's front/rear
  pub gap: Sizon,
  /// one's size
  pub size: f64,
  /// one's front side margin (left/top side)
  pub front_margin: f64,
  /// one's rear side margin (right/bottom side)
  pub rear_margin: f64
}

impl UniPosSize for AbsUniPosSize {
  fn size(&self) -> f64 { self.size }
  fn front_margin(&self) -> f64 { self.front_margin }
  fn rear_margin(&self) -> f64 { self.rear_margin }
}

impl AbsUniPosSize {

  pub fn new(front: bool, outward: bool, gap: Sizon, size: f64, front_margin: f64, rear_margin: f64) -> Self {
    Self {
      front, outward, gap, size, front_margin, rear_margin
    }
  }

  /// Return adjusted front_fixed_pos(`left` or `top`) of an element, 
  /// considering given ancestor's position and size, and document's size
  pub fn front_absolute_to_fixed_pos(
    &self,
    ancestor_front_pos: f64,
    ancestor_size: f64,
    doc_size: f64
  ) -> f64 {

    // get absolute(not relative; in the context of Sizon, not css position) metric gap from sizon.
    // trying order: 1) abs 2) rel 3) fallback return 0.
    let get_actual_gap = move || -> f64 {
      if let Some(abs) = self.gap.abs {
        abs
      } else if let Some(rel) = self.gap.rel.filter(|rel| !rel.is_nan()) {
        ancestor_size * rel
      } else {
        0.
      }
    };

    let get_front_fixed_pos = move |opposite: bool| -> f64 {

      let front = if opposite { !self.front } else { self.front };

      let gap = get_actual_gap();
      let mut key_pos = ancestor_front_pos;
      if !front {
        key_pos += ancestor_size;
      }
      if front==self.outward {
        key_pos -= gap + self.size;
      } else {
        key_pos += gap;
      }
      key_pos
    };

    let is_opposite_better = move || -> bool {
      
      let front_space = ancestor_front_pos - self.front_margin;
      let rear_space = doc_size - front_space - ancestor_size - self.rear_margin;

      if self.front {
        front_space<rear_space && rear_space>self.size
      } else {
        front_space>rear_space && front_space>self.size
      }
    };

    let mut front_fixed_pos = get_front_fixed_pos(false);

    if self.is_over(front_fixed_pos, doc_size) {
      if self.outward {
        if is_opposite_better() {
          front_fixed_pos = get_front_fixed_pos(true);
        }
      }
      self.adjust_front_pos(&mut front_fixed_pos, doc_size);
    }

    front_fixed_pos
  }
}


/// # Fixed Positioned Size
/// 
/// A relevant element is supposed to have style { position: fixed; }
/// 
/// Check out `FixedUniPosSize`
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FixedPosSize {
  lateral: FixedUniPosSize,
  vertical: FixedUniPosSize
}

impl FixedPosSize {

  /// # Args
  /// (size_x, front_margin_x, rear_margin_x): (f64, f64, f64),
  /// 
  /// (size_y, front_margin_y, rear_margin_y): (f64, f64, f64)
  ///
  pub fn new(
    (size_x, front_margin_x, rear_margin_x): (f64, f64, f64),
    (size_y, front_margin_y, rear_margin_y): (f64, f64, f64),
  ) -> Self {
    Self {
      lateral: FixedUniPosSize::new(size_x, front_margin_x, rear_margin_x),
      vertical: FixedUniPosSize::new(size_y, front_margin_y, rear_margin_y)
    }
  }

  /// Return adjusted front_fixed_pos(`left` and `top`) of an element, considering given position and document size
  pub fn front_fixed_pos(&self, (client_x, client_y): (f64, f64)) -> (f64, f64) {
    
    let doc = gloo_utils::document_element();
    let (doc_height, doc_width) = (doc.client_height() as f64, doc.client_width() as f64);

    let fixed_left = self.lateral.front_fixed_pos(client_x, doc_width);
    let fixed_top = self.vertical.front_fixed_pos(client_y, doc_height);
    (fixed_left, fixed_top)
  }

  /// Set style of fixed positions and sizes
  pub fn set_style<H: AsRef<HtmlElement>>(&self, elem: H, client_xy: (f64, f64)) {

    let (fixed_left, fixed_top) = self.front_fixed_pos(client_xy);
    let (width, height) = (self.lateral.size, self.vertical.size);

    let style = elem.as_ref().style();
    let _ = style.set_property("top", &format!("{:.2}px", fixed_top));
    let _ = style.set_property("height", &format!("{:.2}px", height));
    let _ = style.set_property("left", &format!("{:.2}px", fixed_left));
    let _ = style.set_property("width", &format!("{:.2}px", width));
  }
}


/// # Absolute Positioned Size
/// 
/// A relevant element is supposed to have style { position: absolute; }.
/// Thus it's positioned relatively to its relevant ancestor.
/// 
/// Configure the element's relative position to its ancestor:
///   * front: is its position relative to ancestor' left/top or right/bottom?
///   * outward: is the element spreading outwards or inwards in respect to its ancestor?
///   * gap: gap between oneself and ancestor's front/rear. Use `Sizon`.
/// 
/// Check out `AbsUniPosSize`
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AbsPosSize {
  lateral: AbsUniPosSize,
  vertical: AbsUniPosSize
}

impl AbsPosSize {

  /// # Args
  /// (front_x, outward_x, gap_x, size_x, front_margin_x, rear_margin_x): (bool, bool, Sizon, f64, f64, f64),
  /// 
  /// (front_y, outward_y, gap_y, size_y, front_margin_y, rear_margin_y): (bool, bool, Sizon, f64, f64, f64)
  /// 
  /// # Example
  /// ```
  /// # use webtric::*;
  /// 
  /// // This possize refers that the relevant element will be drawn at the position of
  /// // (its ancestor's right pos + 5, ancestors' bottom pos + 5),
  /// // with size of (100, 150), and with margin of (10, 10, 10, 10); 
  /// let abs_possize = AbsPosSize::new(
  ///   (false, true, Sizon::abs(5.), 100., 10., 10.),
  ///   (false, true, Sizon::abs(10.), 150., 10., 10.)
  /// );
  /// ```
  pub fn new(
    (front_x, outward_x, gap_x, size_x, front_margin_x, rear_margin_x): (bool, bool, Sizon, f64, f64, f64),
    (front_y, outward_y, gap_y, size_y, front_margin_y, rear_margin_y): (bool, bool, Sizon, f64, f64, f64)
  ) -> Self {
    Self {
      lateral: AbsUniPosSize::new(front_x, outward_x, gap_x, size_x, front_margin_x, rear_margin_x),
      vertical: AbsUniPosSize::new(front_y, outward_y, gap_y, size_y, front_margin_y, rear_margin_y)
    }
  }

  /// Return adjusted absolute front pos(`left` and `top`) of an element,  
  /// considering given ancestor's position and size, and document's size
  pub fn front_absolute_pos<E: AsRef<Element>>(
    &self,
    ancestor: E
  ) -> (f64, f64) {
    let doc = gloo_utils::document_element();
    let (doc_width, doc_height) = (doc.client_width() as f64, doc.client_height() as f64);

    let (ancestor_top, ancestor_height, ancestor_left, ancestor_width) = get_rect_thlw(ancestor);
    let fixed_left = self.lateral.front_absolute_to_fixed_pos(ancestor_left, ancestor_width, doc_width);
    let fixed_top = self.vertical.front_absolute_to_fixed_pos(ancestor_top, ancestor_height, doc_height); 
    
    (fixed_left-ancestor_left, fixed_top-ancestor_top)
  }


  /// Set style of absolute positions and sizes
  pub fn set_style<E: AsRef<Element>, H: AsRef<HtmlElement>>(&self, ancestor: E, elem: H) {

    let (abs_left, abs_top) = self.front_absolute_pos(ancestor);
    let (width, height) = (self.lateral.size, self.vertical.size);

    let style = elem.as_ref().style();
    let _ = style.set_property("top", &format!("{:.2}px", abs_top));
    let _ = style.set_property("height", &format!("{:.2}px", height));
    let _ = style.set_property("left", &format!("{:.2}px", abs_left));
    let _ = style.set_property("width", &format!("{:.2}px", width));
  }
}