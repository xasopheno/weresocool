extern crate itertools;
extern crate num_rational;
extern crate weresocool;
use itertools::Itertools;
use num_rational::{Ratio, Rational64};

#[derive(Debug, Clone)]
enum Item {
    Collection(Vec<Item>),
    List(Vec<Event>),
}

#[derive(Debug, Clone)]
struct Event {
    s: Sound,
    l: Rational64,
}

#[derive(Debug, Clone)]
struct Sound {
    f: Rational64,
    g: Rational64,
    p: Rational64,
}

impl Sound {
    fn new() -> Sound {
        Sound {
            f: Ratio::new(2, 3),
            g: Ratio::new(6, 7),
            p: Ratio::new(0, 1),
        }
    }
}

impl Event {
    fn new() -> Event {
        Event {
            s: Sound::new(),
            l: Ratio::from_integer(2 / 3),
        }
    }
}

fn render(collection: &[Item]) -> Vec<f32> {
    let mut acc = vec![];
    for item in collection {
        match item {
            &Item::Collection(ref items) => {
                let result = render(items);
                acc = sum_vec(&acc, result)
            }
            &Item::List(ref v) => acc = sum_vec(&acc, event_to_f32(&v.to_vec())),
        }
    }

    acc
}

fn event_to_f32(v: &Vec<Event>) -> Vec<f32> {
    let mut acc = vec![];
    for event in v {
        acc.push(*event.s.f.numer() as f32 / *event.s.f.denom() as f32)
    }

    acc
}

fn sum_vec(a: &Vec<f32>, b: Vec<f32>) -> Vec<f32> {
    let vec_len = std::cmp::max(a.len(), b.len());
    let mut acc: Vec<f32> = vec![0.0; vec_len];
    for (i, e) in a.iter().zip_longest(&b).enumerate() {
        match e {
            itertools::EitherOrBoth::Both(v1, v2) => acc[i] = v1 + v2,
            itertools::EitherOrBoth::Left(e) => acc[i] = *e,
            itertools::EitherOrBoth::Right(e) => acc[i] = *e,
        }
    }

    acc
}

fn update(collection: &mut [Item], fs: &Vec<fn(&mut Vec<Event>)>) {
    for item in collection {
        match *item {
            Item::Collection(ref mut items) => update(items, fs),
            Item::List(ref mut list) => {
                for f in fs {
                    f(list);
                }
            }
        }
    }
}

fn succ_f(list: &mut Vec<Event>) {
    for mut v in list {
        v.s.f += Ratio::from_integer(1);
    }
}

fn succ_g(list: &mut Vec<Event>) {
    for mut v in list {
        v.s.g += Ratio::from_integer(1);
    }
}

fn succ_l(list: &mut Vec<Event>) {
    for mut v in list {
        v.l += Ratio::from_integer(1);
    }
}

fn rational_play() {
    println!("\n\n");

    let a = Ratio::from_float(1.0 / 7.0).unwrap();
    let b = Ratio::from_integer(-2);
    let c = Ratio::from_integer(0);

    println!("{} {} {}", a, b, c);

    let d = Ratio::new(1, 7);
    let e = Ratio::new(3, 2);

    println!("{}", d + e);
    println!("{}", d * e);
    println!("{}", d / e);
    println!("{}", d - e);
}

fn main() {
        use Item::*;

        let mut root = vec![Collection(vec![
            List(vec![Event::new(), Event::new(), Event::new(), Event::new()]),
            Collection(vec![
                List(vec![Event::new(), Event::new(), Event::new()]),
                List(vec![Event::new(), Event::new()]),
            ]),
        ])];

        update(&mut root, &vec![succ_f, succ_g, succ_l]);

        let result = render(&root);
        println!("{:?}", root);
        println!("{:?}", result);

        rational_play()
}

#[cfg(test)]
pub mod tests {
    use self::Item::*;
    use super::*;
    #[test]
    fn test_render() {
        let mut root = vec![Collection(vec![
            List(vec![Event::new(), Event::new(), Event::new(), Event::new()]),
            Collection(vec![
                List(vec![Event::new(), Event::new(), Event::new()]),
                List(vec![Event::new(), Event::new(), Event::new()]),
            ]),
        ])];

        update(&mut root, &vec![succ_f, succ_g, succ_l]);

        let result = render(&root);
        let expected = vec![5.0, 5.0, 5.0, 1.6666666];
        assert_eq!(expected, result);
    }
}
