//! # Webtric = Web + Metric
//! 
//! A wasm library to handle some metric matters of front web environment
//! * Custom **scrollbar** => mod [`scroll`]
//! * **Resizing** parallel panels => mod [`cartons`]
//! * **Reactive positioning** of tooltips or menubars => mod [`possize`]
//! * and [`sizon`]
//! 
//! 
//! ## Features
//! * Feature `sycamore` supports some [sycamore](https://crates.io/crates/sycamore) native functions.
//!   Mind that webtric supports sycamore version of **0.9.0-beta.2** or later. Sycamore has big changes since version 0.9.
//! * ~~Feature `letops` suppoerts some [leptos](https://crates.io/crates/leptos) native functions~~
//!   * It's not yet mature.


pub mod utils;
pub mod error;
use utils::*;
use error::{Error, Result};

#[cfg(feature="sycamore")]
pub use utils::WindowResizing;
#[cfg(feature="leptos")]
pub use utils::LeptosWindowResizing;

pub mod sizon;
pub use sizon::*;

pub mod possize;
pub use possize::*;

pub mod cartons;
pub use cartons::*;

pub mod scroll;
pub use scroll::*;

use std::{cmp::Eq, hash::Hash, str::FromStr};
use hashbrown::{HashSet, HashMap};
pub use rawn::{BoxRaw, BoxRaws};
use serde::{Serialize, Deserialize};
use web_sys::{Element, HtmlElement, Event, WheelEvent, PointerEvent};
use wasm_bindgen::prelude::*;

//#[allow(unused_imports)]
//use gloo_console::log;

#[cfg(feature="sycamore")]
use sycamore::prelude::*;