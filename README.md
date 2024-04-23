# Webtric = Web + Metric

[![Crates.io](https://img.shields.io/crates/v/webtric)](https://crates.io/crates/webtric)
[![docs.rs](https://img.shields.io/docsrs/webtric?label=docs.rs)](https://docs.rs/webtric)

A wasm library to handle some metric matters of front web environment
* Custom **scrollbar** => mod `scroll`
* **Resizing** parallel panels => mod `cartons`
* **Reactive positioning** of tooltips or menubars => mod `possize`
* and `sizon`

See [**the demo page**](https://acheul.github.io/webtric) for live demo.
The page is built on Sycamore. It's src directory is at repository's `/demo-sycamore`.

## Features
* Feature `sycamore` supports some [sycamore](https://crates.io/crates/sycamore) native functions.
  Mind that webtric supports sycamore version of **0.9.0-beta.2** or later. Sycamore has big changes since version 0.9.
  * Check repository's `/demo-sycamore` for sycamore application.
* ~~Feature `letops` suppoerts some [leptos](https://crates.io/crates/leptos) native functions~~
  * It's not yet mature.


## Dev Log
* ver `0.1.0` -> `0.1.1`
  * added feature flags missed in `0.1.0`