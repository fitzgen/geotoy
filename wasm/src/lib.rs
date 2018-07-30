//! My awesome Rust and WebAssembly project!

#![feature(proc_macro, wasm_custom_section, wasm_import_module)]
#![cfg_attr(feature = "wee_alloc", feature(global_allocator))]

#[macro_use]
extern crate cfg_if;

cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        fn set_panic_hook() {}
    }
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate geotoy;

use geotoy::{Attractor, Kind, Point};
use std::cell::RefCell;

thread_local! {
    static MESH: RefCell<Option<(Vec<Point>, Vec<u16>, Vec<u16>, Vec<Attractor>, Vec<Kind>)>> =
        RefCell::new(None);
}

#[wasm_bindgen]
pub fn create_mesh(rows: usize, columns: usize, size: f64) {
    set_panic_hook();
    MESH.with(|mesh| mesh.replace(Some(geotoy::mesh(rows, columns, size as f32))));
}

#[wasm_bindgen]
pub fn points_len() -> usize {
    MESH.with(|mesh| mesh.borrow().as_ref().unwrap().0.len())
}

#[wasm_bindgen]
pub fn point_dim() -> usize {
    2
}

#[wasm_bindgen]
pub fn points() -> *const Point {
    MESH.with(|mesh| mesh.borrow().as_ref().unwrap().0.as_ptr())
}

#[wasm_bindgen]
pub fn lines_len() -> usize {
    MESH.with(|mesh| mesh.borrow().as_ref().unwrap().1.len())
}

#[wasm_bindgen]
pub fn line_dim() -> usize {
    1
}

#[wasm_bindgen]
pub fn lines() -> *const u16 {
    MESH.with(|mesh| mesh.borrow().as_ref().unwrap().1.as_ptr())
}

#[wasm_bindgen]
pub fn triangles_len() -> usize {
    MESH.with(|mesh| mesh.borrow().as_ref().unwrap().2.len())
}

#[wasm_bindgen]
pub fn triangle_dim() -> usize {
    1
}

#[wasm_bindgen]
pub fn triangles() -> *const u16 {
    MESH.with(|mesh| mesh.borrow().as_ref().unwrap().2.as_ptr())
}

#[wasm_bindgen]
pub fn attractors_len() -> usize {
    MESH.with(|mesh| mesh.borrow().as_ref().unwrap().3.len())
}

#[wasm_bindgen]
pub fn attractor_dim() -> usize {
    2
}

#[wasm_bindgen]
pub fn attractors() -> *const Attractor {
    MESH.with(|mesh| mesh.borrow().as_ref().unwrap().3.as_ptr())
}

#[wasm_bindgen]
pub fn kinds_len() -> usize {
    MESH.with(|mesh| mesh.borrow().as_ref().unwrap().4.len())
}

#[wasm_bindgen]
pub fn kind_dim() -> usize {
    1
}
#[wasm_bindgen]
pub fn kinds() -> *const Kind {
    MESH.with(|mesh| mesh.borrow().as_ref().unwrap().4.as_ptr())
}

#[wasm_bindgen]
pub fn vertex_shader() -> String {
    geotoy::VERTEX_SHADER_WEB.into()
}

#[wasm_bindgen]
pub fn fragment_shader() -> String {
    geotoy::FRAGMENT_SHADER_WEB.into()
}

// #[wasm_bindgen(module = "./gl.js")]
// extern {
//     #[wasm_bindgen(js_name = "getContext")]
//     fn get_context() -> WebGLRenderingContext;
// }

// #[wasm_bindgen]
// extern {
//     type WebGLRenderingContext;

//     #[wasm_bindgen(method, js_name = "createBuffer")]
//     fn create_buffer(&WebGLRenderingContext) -> WebGLBuffer;
// }

// #[wasm_bindgen]
// extern {
//     type WebGLBuffer;
// }
