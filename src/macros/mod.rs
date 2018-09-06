macro_rules! r {
    ($(($num:expr,$den:expr,$offset:expr,$gain:expr,$pan:expr)),*$(,)*) => {
        Op::Overlay { operations: vec![$(
                Op::Compose { operations: vec! [
                    Op::TransposeM { m: $num as f32 /$den as f32 },
                    Op::TransposeA { a: $offset },
                    Op::Gain { m: $gain },
                    Op::PanA { a: $pan },
                ]},
            )*]
        }
    }
}

macro_rules! compose {
    ( $( $operation:expr ),*$(,)* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($operation);
            )*

            Op::Compose { operations: temp_vec }
        };
    };
}

macro_rules! sequence {
    ( $( $operation:expr ),*$(,)* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($operation);
            )*
            Op::Sequence { operations: temp_vec }
        };
    };
}

macro_rules! overlay {
    ( $( $operation:expr ),*$(,)* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($operation);
            )*
            Op::Overlay { operations: temp_vec }
        };
    };
}

macro_rules! fit {
    ($main:expr => $with_length_of:expr, $n:expr) => {
        Op::Fit {
            n: $n,
            with_length_of: Box::new($with_length_of),
            main: Box::new($main),
        }
    };
}

macro_rules! repeat {
    ($operation:expr, $n:expr) => {
        Op::Repeat {
            n: $n,
            operations: {vec![$operation]}
        }
    };
}

#[cfg(test)]
mod test;
