macro_rules! r {
    ($($val1:expr,$val2:expr,$val3:expr,$val4:expr,$pan:ident)*) => {
        $( R::atio($val1,$val2,$val3,$val4,pan_expand!($pan)))*
    }
}
macro_rules! pan_expand {
    (left) => {Pan::Left};
    (right) => {Pan::Right};
    ($other:tt) => {$other};
}


