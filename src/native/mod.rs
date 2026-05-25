macro_rules! m {
    ($ident:ident) => {
        mod $ident;
        pub use $ident::*;
    };
}

#[cfg(unix)]
m!(unix);

#[cfg(windows)]
m!(windows);
