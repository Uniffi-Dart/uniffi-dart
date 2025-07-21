/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::time::{Duration, Instant};

pub struct TestData {
    pub foo: String,
    pub bar: String,
}

pub enum TestCase {
    Function,
    VoidReturn,
    NoArgsVoidReturn,
}

pub trait TestCallbackInterface: Send + Sync {
    fn method(&self, a: i32, b: i32, data: TestData) -> String;
    fn method_with_void_return(&self, a: i32, b: i32, data: TestData);
    fn method_with_no_args_and_void_return(&self);
    fn run_test(&self, test_case: TestCase, count: u64) -> u64;
}

// Test functions for benchmarking FFI call overhead
pub fn test_function(_a: i32, _b: i32, data: TestData) -> String {
    data.bar
}

pub fn test_void_return(_a: i32, _b: i32, _data: TestData) {
    // Intentionally does nothing - testing void return overhead
}

pub fn test_no_args_void_return() {
    // Intentionally does nothing - testing minimal call overhead
}

pub fn run_benchmarks(language: String, cb: Box<dyn TestCallbackInterface>) {
    println!("Running benchmarks for {language}");
    
    let test_data = TestData {
        foo: "SomeStringData".to_string(),
        bar: "SomeMoreStringData".to_string(),
    };
    
    // Simple timing-based benchmarks (not using Criterion for now to avoid complexity)
    
    // Test function calls
    let start = Instant::now();
    for _ in 0..1000 {
        test_function(10, 100, TestData {
            foo: test_data.foo.clone(),
            bar: test_data.bar.clone(),
        });
    }
    let function_time = start.elapsed();
    println!("{language}-functions-basic: {:?}", function_time);
    
    // Test void return
    let start = Instant::now();
    for _ in 0..1000 {
        test_void_return(10, 100, TestData {
            foo: test_data.foo.clone(),
            bar: test_data.bar.clone(),
        });
    }
    let void_time = start.elapsed();
    println!("{language}-functions-void-return: {:?}", void_time);
    
    // Test no-args void return
    let start = Instant::now();
    for _ in 0..1000 {
        test_no_args_void_return();
    }
    let no_args_time = start.elapsed();
    println!("{language}-functions-no-args-void-return: {:?}", no_args_time);
    
    // Test callbacks
    let start = Instant::now();
    for _ in 0..1000 {
        cb.method(10, 100, TestData {
            foo: test_data.foo.clone(),
            bar: test_data.bar.clone(),
        });
    }
    let callback_time = start.elapsed();
    println!("{language}-callbacks-basic: {:?}", callback_time);
    
    // Test callback void return
    let start = Instant::now();
    for _ in 0..1000 {
        cb.method_with_void_return(10, 100, TestData {
            foo: test_data.foo.clone(),
            bar: test_data.bar.clone(),
        });
    }
    let callback_void_time = start.elapsed();
    println!("{language}-callbacks-void-return: {:?}", callback_void_time);
    
    // Test callback no args void return
    let start = Instant::now();
    for _ in 0..1000 {
        cb.method_with_no_args_and_void_return();
    }
    let callback_no_args_time = start.elapsed();
    println!("{language}-callbacks-no-args-void-return: {:?}", callback_no_args_time);
    
    println!("Benchmarks complete for {language}!");
}

uniffi::include_scaffolding!("api"); 