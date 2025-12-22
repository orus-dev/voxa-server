#[macro_export]
macro_rules! logger {
    (const $i:ident $name:expr) => {
        #[allow(unused)]
        pub const $i: $crate::once_cell::sync::Lazy<$crate::utils::logger::Logger> =
            $crate::once_cell::sync::Lazy::new(|| $crate::utils::logger::Logger::new($name));
    };

    ($i:ident $name:expr) => {
        #[allow(unused)]
        pub const $i: $crate::once_cell::sync::Lazy<$crate::utils::logger::Logger> =
            $crate::once_cell::sync::Lazy::new(|| $crate::utils::logger::Logger::new($name));
    };

    (const $name:expr) => {
        $crate::once_cell::sync::Lazy::new(|| $crate::utils::logger::Logger::new($name))
    };

    ($name:expr) => {
        $crate::logger::utils::Logger::new($name)
    };
}
