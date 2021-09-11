const INPUT_NUM: usize = 10;

// const N: usize = 16; // NxN 区画
// const M: usize = 5000; // 野菜の数 M
// const T: usize = 1000; // 行動日数

fn main() {
    let input_path = "A/tester".to_string();

    let mut inputs = vec![];
    for i in 0..INPUT_NUM {
        let file_path = format!("{}/input_{}.txt", input_path, i);
        inputs.push(read_file(file_path));
    }

    // inputs内の要素に対して処理を書く
    eprintln!("{}", inputs.len());
}

#[allow(dead_code, unused)]
fn read_file(file_path: String) -> Input {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    let file = File::open(file_path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_line(&mut String::new());
    buf_reader.read_to_string(&mut contents);

    let mut vegets = vec![];
    for s in contents.split("\n") {
        if s.len() == 0 {
            break;
        }
        let v = s
            .split(" ")
            .map(|e| e.parse::<usize>().unwrap())
            .collect::<Vec<_>>();
        vegets.push(Vegetable {
            pos: Coord::from_usize_pair((v[1], v[0])),
            s_day: v[2],
            e_day: v[3],
            value: v[4],
        });
    }

    Input { vegets }
}

#[derive(Clone)]
struct Input {
    vegets: Vec<Vegetable>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Coord {
    x: isize,
    y: isize,
}

#[derive(Clone, PartialEq, Eq)]
struct Vegetable {
    pos: Coord,
    s_day: usize, // s_day の頭に生える
    e_day: usize, // e_day の最後に消える
    value: usize,
}

#[allow(dead_code)]
impl Coord {
    fn new(p: (isize, isize)) -> Self {
        Coord { x: p.0, y: p.1 }
    }
    fn from_usize_pair(p: (usize, usize)) -> Self {
        Coord {
            x: p.0 as isize,
            y: p.1 as isize,
        }
    }
}
