## iOS Integration via XCFramework

This project now builds the Rust layer (`xilem_abcdeez_mobile`) into an XCFramework that you add to Xcode.

### Build

From `xilem-cross-platform/`:

```sh
./make_xcframework.sh          # Debug
./make_xcframework.sh --release
```

Artifacts:

- Static libs in `../target/aarch64-apple-ios{,-sim}/{debug,release}`
- Mirrored to `./target/...` for legacy scripts
- XCFramework at `dist/xilem_abcdeez_mobile.xcframework`

### Add to Xcode

Option A (recommended): Add a local SPM package with a Binary Target pointing to `dist/xilem_abcdeez_mobile.xcframework`.

Option B: Drag `dist/xilem_abcdeez_mobile.xcframework` into the project and set Embed to "Do Not Embed".

Remove the legacy Cargo build phase/target ("cargo_ios"). Xcode will only link the XCFramework.

### Notes

- Embedded backend is HTTP-only; app starts it on 127.0.0.1:3000.
- For changes in Rust, re-run the script and rebuild in Xcode.

### Keep Xcode’s Run button in sync (optional)

If you want Xcode to auto-refresh the XCFramework when Rust sources change, add a pre-build Run Script phase to your app target that calls:

```
${SRCROOT}/xilem-cross-platform/ios-prebuild-rust.sh
```

This script hashes Rust sources and only runs `make_xcframework.sh` when something changed and the configuration (Debug/Release) requires a different build.

# xilem cross-platform

Example setup for using xilem on android, iOS, and desktop

### Instructions

## iOS

> Note: may not work on the Apple iOS Simulator due to lack of capabilites
> [\*](https://developer.apple.com/documentation/metal/developing_metal_apps_that_run_in_simulator#3241608)

```sh
rustup target add aarch64-apple-ios

open xilem_abcdeez_mobile.xcodeproj
# Configure signing

# Build
```

### Android

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi
cargo install cargo-apk

cargo apk run -p xilem_abcdeez_mobile
```

### Desktop

> Note: only tested on macOS but should work on other systems as well

```sh
cargo run --manifest-path=./desktop/Cargo.toml
```

## Architecture

- The android build uses [cargo-apk](https://github.com/rust-mobile/cargo-apk)
  which, although it's deprecated, is the most stable way of building rust
  projects into android apps. It's configured in `mobile/Cargo.toml` and
  calls the `android_main` function from the `mobile` crate.
- iOS also uses the `mobile` crate but compiles is a library(using `cdylib`)
  which is then called by an Objective-C file in `ios-src`. There's an XCode
  project in `xilem_abcdeez_mobile.xcodeproj` that includes the required files
  and which calls `build_rust_deps.sh` during build to compile the rust code.
- There's a separate crate for the desktop build since cargo-apk fails if the
  mobile project contains a binary target. It's a normal cargo project
- The application logic is a separate project so that it can be imported
  by both the mobile and desktop crates. It's based on the xilem mason example.

The setup should work with winit directly and would only require removing xilem
from the app crate and instead creating the event loop and window directly.

## References

- https://github.com/bevyengine/bevy/tree/main/examples/mobile
- https://github.com/ryanmcgrath/cacao/tree/trunk/examples/ios-beta
- https://github.com/linebender/vello#android
