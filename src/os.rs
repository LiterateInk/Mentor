pub fn dylib_or_so() -> &'static str {
  if cfg!(target_os = "macos") {
    "dylib"
  } else {
    "so"
  }
}
