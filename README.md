# Mentor

A specialized CLI that helps our GitHub Actions handle heavy tasks with a single command.

## Installation

```bash
cargo install --git https://github.com/LiterateInk/Mentor
```

## Setup

This is centered to projects using the [`uniffi`](https://github.com/mozilla/uniffi-rs) library, we recommend using the `git` version for latest fixes, especially concerning Swift 6, even though it's not stable yet.

You might inspire on our repositories, such as [Scodok](https://github.com/LiterateInk/Scodok), to understand how this CLI is used, especially in the CI/CD pipeline.

## Commands

### `bump <patch|minor|major>`

Bumps the version according to SemVer for every language.

- `rust`: `Cargo.toml` (current version is read from there)
- `swift`: `Package.swift`
- `kotlin`: `kotlin/build.gradle.kts`
- `js`: `package.json` (will also bump dependencies, you need `pnpm` installed)

> In the future, it should also update all instances in the `README.md` to reflect the new version.

### `make-swift` (only on macOS)

First, it builds the library for the following targets :

- `aarch64-apple-ios`
- `aarch64-apple-ios-sim`
- `x86_64-apple-ios`
- `aarch64-apple-darwin`
- `x86_64-apple-darwin`

> Make sure you have them enabled - if you don't, you can run `rustup target add <target>` to add each of them.

Then, it merges `aarch64-apple-ios-sim` and `x86_64-apple-ios` to produce the `aarch64-x86_64-apple-ios-sim` library - intended for the iOS simulator. It also merges `aarch64-apple-darwin` and `x86_64-apple-darwin` to produce the `aarch64-x86_64-apple-darwin` library  - intended for macOS.

> This is done using `lipo`.

Then, it generates the `swift/<lib-name>.swift` file with the latest bindings.

> `lib-name` is taken from the `package.name` property in `Cargo.toml` file.

Finally, it creates the `xcframework` for the library in `target/<lib-name>FFI.xcframework`.

> This is done using `xcodebuild`.

And it will also generate a `zip` file at `target/<lib-name>FFI.xcframework.zip`, for distribution.

> This is done using `ditto`.

Finally, it will update the checksum in the `Package.swift` file, using the `zip` file generated.

> This is done using `shasum -a 256`.

### `make-kotlin` (only on Linux or macOS, since easier)

Make sure you have the following targets :

- `x86_64-unknown-linux-gnu`
- `aarch64-linux-android`
- `armv7-linux-androideabi`
- `i686-linux-android`
- `x86_64-linux-android`

> If you don't, you can run `rustup target add <target>` to add each of them.

Also, you should have `cargo-ndk` installed.

Firstly, it builds the library using the machine's current target to generate Kotlin bindings - outputs to `kotlin/src/commonMain/kotlin`.

Finally, it builds the library for all the Android targets mentioned above, using `cargo-ndk` - outputs to `kotlin/src/androidMain/jniLibs`.

Sadly, we don't have support for other platforms that Kotlin Multiplatform supports, such as desktop and iOS.
Maybe in the future ?

### `ci-push <prepare|swift|release-tag>`

A small utility to commit and push the changes to the repository from GitHub Actions.

- `prepare` : `chore: bump version to v<version> with latest dependencies`
- `swift` : `chore: update swift bindings and checksum for v<version>`
- `release-tag` : creates a tag `<version>` and pushes it to the repository

### `make-js`

Make sure you have `wasm-pack` installed - you can install it using `cargo install wasm-pack`.
You also need `terser` installed for minification - you can install it using `npm install -g terser`.

Output will be available in the `js` directory.
You can directly publish the package to `npm` after running this command.

### `version`

Prints the current version of the library - reads from the `Cargo.toml` file.

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.
