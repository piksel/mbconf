#[macro_export]
macro_rules! pack {
    ( $($b:expr),* ) => {
        (
            Buf::<0>::new()
            $(
                + Buf::from($b)
            )*
        ).into_bytes()
    };
}