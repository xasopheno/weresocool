extern crate weresocool;

fn main() {
    println!("{}", "
        ___ Base Operations ___
        AsIs
        Reverse
        Repeat
        Silence <n>
        Tm <n>
        Ta <n>
        PanM <n>
        PanA <n>
        Length <n>
        Gain <n>

        ___ Grouping Operations ___
        Sequence [ op1, op2, op3, etc ]
        Overlay [ op1, op2, op3, etc ]

        ___ Overtones ___
        O[
            (3/2, 0.0, 0.5, -0.5),
            (1/1, 5.0, 0.5, 0.5),
        ]

        ___ Variables ___
        thing = {
            Sequence [AsIs, Tm 3/2]
        }

        ___ Compose ___
        thing2 = {
            thing1
            | Tm 5/4
            | Sequence [AsIs, Tm 3/2]
            | Gain 0.75
        }
        ___ Fit ___
        main = {
            Overlay [
                thing1,
                thing2
                | PanA 0.5
                > FitLength 0.5
            ]
        }
    ")
}
