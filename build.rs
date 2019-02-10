use cmake::Config;
use std::{env, path::PathBuf};

fn main() {
    if cfg!(feature = "build-nng") {
        let dst = cmake();
        // Check output of `cargo build --verbose`, should see something like:
        // -L native=/path/runng/target/debug/build/runng-sys-abc1234/out
        // That contains output from cmake
        println!(
            "cargo:rustc-link-search=native={}",
            dst.join("lib").display()
        );
        println!(
            "cargo:rustc-link-search=native={}",
            dst.join("lib64").display()
        );

        // Tell rustc to use nng static library
        println!("cargo:rustc-link-lib=static=nng");
    } else {
        println!("cargo:rustc-link-lib=dylib=nng");
    }

    // https://rust-lang-nursery.github.io/rust-bindgen
    // https://docs.rs/bindgen
    let mut builder = bindgen::Builder::default()
        // This is needed if use `#include <nng.h>` instead of `#include "path/nng.h"` in wrapper.h
        //.clang_arg("-Inng/src/")
        .header("wrapper.h");

    if !cfg!(feature = "legacy-111-rc4") {
        builder = builder
            // #[derive(Default)]
            .derive_default(true)
            .whitelist_type("nng_.*")
            .whitelist_function("nng_.*")
            .whitelist_var("NNG_.*")
            // Generate `pub const NNG_UNIT_EVENTS` instead of `nng_unit_enum_NNG_UNIT_EVENTS`
            .prepend_enum_name(false)
            // Generate `pub enum ...` instead of multiple `pub const ...`
            .rustified_enum("nng_.*_enum")
            // Enum special cases:
            .rustified_enum("nng_pipe_ev")
            .rustified_enum("nng_sockaddr_family")
            .rustified_enum("nng_zt_status")
            .use_core();
    }
    if cfg!(feature = "no_std") {
        // no_std support
        // https://rust-embedded.github.io/book/interoperability/c-with-rust.html#automatically-generating-the-interface
        builder = builder.ctypes_prefix("cty")
    }
    let bindings = builder.generate().expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}

fn cmake() -> PathBuf {
    // Compile-time features
    let generator = if cfg!(feature = "cmake-ninja") {
        "Ninja"
    } else if cfg!(feature = "cmake-vs2017") {
        "Visual Studio 15 2017"
    } else if cfg!(feature = "cmake-vs2017-win64") {
        "Visual Studio 15 2017 Win64"
    } else {
        // Default generators
        if cfg!(target_family = "unix") {
            "Unix Makefiles"
        } else {
            "Ninja"
        }
    };
    let stats = if cfg!(feature = "nng-stats") {
        "ON"
    } else {
        "OFF"
    };
    let tls = if cfg!(feature = "nng-tls") {
        "ON"
    } else {
        "OFF"
    };

    // Run cmake to build nng
    Config::new("nng")
        .generator(generator)
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("NNG_TESTS", "OFF")
        .define("NNG_TOOLS", "OFF")
        .define("NNG_ENABLE_STATS", stats)
        .define("NNG_ENABLE_TLS", tls)
        .build()
}
