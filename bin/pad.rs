extern crate itertools;
extern crate weresocool;
use itertools::Itertools;

#[derive(Debug)]
enum Item {
    Collection(Vec<Item>),
    List(Vec<Event>),
}

#[derive(Debug)]
struct Event {
    s: Sound,
    l: f32
}

#[derive(Debug)]
struct Sound {
    f: f32,
    g: f32,
    p: f32
}

impl Sound {
    fn new() -> Sound {
        Sound {
            f: 10.0,
            g: 1.0,
            p: 0.0
        }
    }
}

impl Event {
    fn new() -> Event {
        Event {
            s: Sound::new(),
            l: 1.0
        }
    }
}

fn render(collection: &[Item]) -> Vec<usize> {
    let mut acc = vec![];
    for item in collection {
        println!("{:?}", item);
        match item {
            &Item::Collection(ref items) => {
                let result = render(items);
                acc = sum_vec(&acc, result)
            }
            &Item::List(ref v) => acc = sum_vec(&acc, v.to_vec()),
        }
    }

    acc
}

//fn sum_vec(a: &Vec<usize>, b: Vec<usize>) -> Vec<usize> {
//    let vec_len = std::cmp::max(a.len(), b.len());
//    let mut acc: Vec<usize> = vec![0; vec_len];
//    for (i, val) in a.iter().zip_longest(&b).enumerate() {
//        match val {
//            itertools::EitherOrBoth::Both(v1, v2) => acc[i] = v1 + v2,
//            itertools::EitherOrBoth::Left(v) => acc[i] = *v,
//            itertools::EitherOrBoth::Right(v) => acc[i] = *v
//        }
//    }
//
//    acc
//}

fn update(collection: &mut [Item], fs: &Vec<fn(&mut Vec<usize>)>) {
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

fn add_one(list: &mut Vec<usize>) {
    for mut v in list {
        *v += 1;
    }
}


fn main() {
    use Item::*;

    let mut root = vec![
        Collection(vec![
            List(vec![Event::new(), Event::new(), Event::new(), Event::new()]),
            Collection(vec![
                List(vec![Event::new(), Event::new(), Event::new()]),
                List(vec![Event::new(), Event::new()]),
            ]),
        ]),
    ];

//    let result = render(&root);
//    println!("{:?}", result);


//    for _i in 0..4 {
    update(&mut root, &vec![add_one]);
//    }


//    let result = render(&root);
    println!("{:?}", root);
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use self::Item::*;
    #[test]
    fn test_render() {
        let mut root = vec![
            Collection(vec![
                List(vec![Event::new(), Event::new(), Event::new(), Event::new()]),
                Collection(vec![
                    List(vec![Event::new(), Event::new(), Event::new()]),
                    List(vec![Event::new(), Event::new()]),
                ]),
            ]),
        ];

//        let result = render(&root);

        let expected = vec![3, 3, 0, 3, 1, 0, 0, 1, 1];
        assert_eq!(expected, result);
    }

    #[test]
    fn test_inner_long_render() {
        let root = vec![
            Collection(vec![
                List(vec![1, 1, 0, 1]),
                Collection(vec![
                    List(vec![1, 1, 0, 1]),
                    List(vec![1, 1, 0, 1, 1, 0, 0, 1, 1]),
                ]),
                Collection(vec![
                    List(vec![1, 1, 0, 1]),
                    List(vec![1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1]),
                ]),
                List(vec![0, 1, 1, 0, 1]),
            ]),
        ];

        let result = render(&root);

        let expected = vec![5, 6, 1, 5, 3, 0, 0, 2, 2, 0, 0, 0, 1];
        assert_eq!(expected, result);
    }
}
