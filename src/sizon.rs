//! # Sizon = Size + -on
//! 
//! In the web's front environment, a length of an object can be expressed and handled in both of relative(%) or absolute(px) units.
//! Usually it's a trivial problem, but sometimes quite cumbersome to take care of.
//! 
//! For example, a minimum or maximum limitation of a changeable size can be decided in both of relative or absolute aspects.
//! Then we need to keep information of both of them. It would be better if we can handle them with a simple data structure.
//! 
//! Sizon itself is a very small struct to handle this *size duality* issue.
//! It consolidates the relativeness and the absoluteness.
//! 
//! It's named to be feel like particulate, something like boson, grviton, uhuh ... huh.. size entanglement?!

use crate::*;

/// Sizon has just two fields: `abs` and `rel`.
/// 
/// `abs` would refer to in-pixel size while `rel` would refer to relative-to-parent(ancestor) size ratio.
/// 
/// Mind that `rel` is **not** supposed to be percent(%). Just a pure ratio.
/// 
/// Recommend to use default value of it for something like `None`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct Sizon {
  /// absolute size in pixel
  pub abs: Option<f64>,
  /// relative size raito
  pub rel: Option<f64>,
}

impl Default for Sizon {
  /// Default value of Sizon can be used like `None`
  fn default() -> Self {
    Self {
      abs: None, rel: None
    }
  }
}

impl Sizon {

  /// return new Sizon
  pub fn new(abs: Option<f64>, rel: Option<f64>) -> Self {
    Self { abs, rel }
  }

  /// return new Sizon with `abs` value
  pub fn abs(abs: f64) -> Self {
    Self { abs: Some(abs), rel: None }
  }

  /// return new Sizon with `rel` value
  pub fn rel(rel: f64) -> Self {
    Self { abs: None, rel: Some(rel) }
  }

  /// Just like rust's native `max` method, but using self's `abs` field.
  /// If `abs` field is none, just return the given value.
  /// 
  /// # Example
  /// ```
  /// # use webtric::Sizon;
  /// let sizon = Sizon::abs(20.);
  /// let other = 30.;
  /// assert!(sizon.max_abs(other)==other);
  /// ```
  pub fn max_abs(&self, abs: f64) -> f64 {
    self.abs.map(|abs_| abs_.max(abs)).unwrap_or(abs)
  }

  /// Just like rust's native `min` method, but using self's `abs` field.
  /// If `abs` field is none, just return the given value.
  /// 
  /// # Example
  /// ```
  /// # use webtric::Sizon;
  /// let sizon = Sizon::abs(20.);
  /// let other = 30.;
  /// assert!(sizon.min_abs(other)==20.);
  /// ```
  pub fn min_abs(&self, abs: f64) -> f64 {
    self.abs.map(|abs_| abs_.min(abs)).unwrap_or(abs)
  }

  /// Just like rust's native `max` method, but using self's `rel` field.
  /// If `rel` field is none, just return the given value.
  /// 
  /// # Example
  /// ```
  /// # use webtric::Sizon;
  /// let sizon = Sizon::rel(0.2);
  /// let other = 0.3;
  /// assert!(sizon.max_rel(other)==other);
  /// ```
  pub fn max_rel(&self, rel: f64) -> f64 {
    self.rel.map(|rel_| rel_.max(rel)).unwrap_or(rel)
  }

  /// Just like rust's native `min` method, but using self's `rel` field.
  /// If `rel` field is none, just return the given value.
  /// 
  /// # Example
  /// ```
  /// # use webtric::Sizon;
  /// let sizon = Sizon::rel(0.2);
  /// let other = 0.3;
  /// assert!(sizon.min_rel(other)==0.2);
  /// ```
  pub fn min_rel(&self, rel: f64) -> f64 {
    self.rel.map(|rel_| rel_.min(rel)).unwrap_or(rel)
  }

  /// Just like rust's native `max` method, using both of self's `abs` and `rel` fields.
  /// 
  /// It tries to use `rel` field only when parent's size(`par`) argument is Some.
  /// When self's `rel` field is bigger than ratio of argument `abs` to argument `par`,
  /// it returns value of argument `par` multiplied by self's `rel` field.
  /// 
  /// # Example
  /// ```
  /// # use webtric::Sizon;
  /// let sizon = Sizon::new(Some(20.), Some(0.5));
  /// assert!(sizon.max(30., None)==30.);
  /// assert!(sizon.max(30., Some(100.))==50.);
  /// ```
  pub fn max(&self, abs: f64, par: Option<f64>) -> f64 {
    let abs = self.max_abs(abs);
    if let Some(rel) = self.rel {
      if let Some(par) = par.filter(|par| par.is_normal()) {
        if rel>abs/par {
          return par*rel; 
        }
      }
    }
    abs
  }

  /// Just like rust's native `min` method, using both of self's `abs` and `rel` fields.
  /// 
  /// It tries to use `rel` field only when parent's size(`par`) argument is some.
  /// When self's `rel` field is smaller than ratio of argument `abs` to argument `par`,
  /// it returns value of argument `par` multiplied by self's `rel` field.
  /// 
  /// # Example
  /// ```
  /// # use webtric::Sizon;
  /// let sizon = Sizon::new(Some(20.), Some(0.1));
  /// assert!(sizon.min(30., None)==20.);
  /// assert!(sizon.min(30., Some(100.))==10.);
  /// ```
  pub fn min(&self, abs: f64, par: Option<f64>) -> f64 {
    let abs = self.min_abs(abs);    
    if let Some(rel) = self.rel {
      if let Some(par) = par.filter(|par| par.is_normal()) {
        if rel<abs/par {
          return par*rel;
        }
      }
    }
    abs
  }

  /// Build a new Sizon from a given element.
  /// Use element's and its parent's DomRect size.
  pub fn elem<E: AsRef<Element>>(elem: E, lateral: bool) -> Self {

    let abs = get_elem_size(elem.as_ref(), lateral);
    let par = get_elem_size(elem.as_ref().parent_element().unwrap_or(gloo_utils::document_element()), lateral);

    let rel = if par.is_normal() { Some(abs/par) } else { None };

    Self { abs: Some(abs), rel }
  }

  /// Returns size style formated value. Multiplies 100 for `rel` value.
  /// Ex. with Sizon { abs: 20., rel: 0.2 },
  /// when `abs` is true, returns "20px" literal, 
  /// while `abs` is false, returns "20%" literal.
  pub fn style_value(&self, abs: bool) -> Option<String> {
    if abs {
      self.abs.map(|abs| format!("{:.2}px", abs))
    } else {
      // Make sure to multiply 100 so as to be percent ratio.
      self.rel.map(|rel| format!("{:.2}%", rel*100.))
    }
  }

  /// set size style property using `abs` or `rel`
  pub fn set_style<E: AsRef<HtmlElement>>(&self, elem: E, abs: bool, lateral: bool) -> bool {
    
    if let Some(value) = self.style_value(abs) {
      let property = if lateral {"width"} else {"height"};
      elem.as_ref().clone().style().set_property(property, value.as_str()).unwrap_throw();
      true
    } else {
      false
    }
  }

  /// Get absolute value from given parent's size value(`par`).
  /// field `abs` has higher priority to be returned.
  pub fn to_abs(&self, par: f64) -> Option<f64> {
    if let Some(abs) = self.abs {
      Some(abs)
    } else if let Some(rel) = self.rel.filter(|rel| !rel.is_nan()) {
      Some(par*rel)
    } else {
      None
    }
  }
}