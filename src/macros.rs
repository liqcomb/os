macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!(&mut ::debug::SerialWriter::get(module_path!(), true), $($arg)*);
    })
}

macro_rules! printf {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!(&mut ::debug::SerialWriter::get(module_path!(), false), $($arg)*);
    })
}

macro_rules! err {
    ($x:ident) => {{
        use common::error::Error;
        Error::$x
    }};
}

macro_rules! align {
    ($x:expr, $align:expr) => {{
        ($x + $align - 1) & (!($align - 1))
    }};
}
