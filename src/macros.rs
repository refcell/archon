/// This macro is used to read the value of an environment variable.
/// If the environment variable is not set, the macro will panic.
#[macro_export]
macro_rules! extract_env {
    ($a:expr) => {
        std::env::var($a).unwrap_or_else(|_| panic!("{} is not set", $a))
    };
}
