#[allow(unused_imports)]
use proconio::marker::{Chars, Isize1, Usize1};
use proconio::{fastout, input};

#[allow(unused_imports)]
use std::cmp::*;
#[allow(unused_imports)]
use std::collections::*;
#[allow(unused_imports)]
use std::io::Write;

#[allow(unused_imports)]
use rand::rngs::ThreadRng;
#[allow(unused_imports)]
use rand::seq::SliceRandom;
#[allow(unused_imports)]
use rand::{thread_rng, Rng};
use std::time::SystemTime;

#[allow(dead_code)]
const MOD: usize = 1e9 as usize + 7;

const N: usize = 16; // NxN 区画
const M: usize = 5000; // 野菜の数 M
const T: usize = 1000; // 行動日数

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Coord {
    x: isize,
    y: isize,
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

    fn in_field(&self) -> bool {
        (0 <= self.x && self.x < N as isize) && (0 <= self.y && self.y < N as isize)
    }

    // ペアへの変換
    fn to_pair(&self) -> (isize, isize) {
        (self.x, self.y)
    }

    // マンハッタン距離
    fn distance(&self, that: &Self) -> isize {
        (self.x - that.x).abs() + (self.y - that.y).abs()
    }

    fn mk_4dir(&self) -> Vec<Self> {
        let delta = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        delta
            .iter()
            .map(|&p| self.plus(&Coord::new(p)))
            .filter(|&pos| pos.in_field())
            .collect()
    }

    fn com_to_delta(com: char) -> Self {
        match com {
            'U' => Coord::new((0, -1)),
            'D' => Coord::new((0, 1)),
            'L' => Coord::new((-1, 0)),
            'R' => Coord::new((1, 0)),
            _ => unreachable!(),
        }
    }

    // 四則演算
    fn plus(&self, that: &Self) -> Self {
        Coord::new((self.x + that.x, self.y + that.y))
    }
    fn minus(&self, that: &Self) -> Self {
        Coord::new((self.x - that.x, self.y - that.y))
    }

    fn access_matrix<'a, T>(&'a self, mat: &'a Vec<Vec<T>>) -> &'a T {
        &mat[self.y as usize][self.x as usize]
    }

    fn set_matrix<T>(&self, mat: &mut Vec<Vec<T>>, e: T) {
        mat[self.y as usize][self.x as usize] = e;
    }
}

#[derive(Clone)]
struct Vegetable {
    pos: Coord,
    s_day: usize, // s_day の頭に生える
    e_day: usize, // e_day の最後に消える
    value: usize,
}

struct Input {
    vegets: Vec<Vegetable>,
}
impl Input {
    fn new(rcsev: Vec<(usize, usize, usize, usize, usize)>) -> Self {
        let mut vegets = vec![];
        for (r, c, s, e, v) in rcsev {
            let veget = Vegetable {
                pos: Coord::from_usize_pair((c, r)),
                s_day: s,
                e_day: e,
                value: v,
            };
            vegets.push(veget);
        }
        Input { vegets }
    }
}

#[derive(Clone)]
enum Command {
    Buy(Coord),
    Move((Coord, Coord)),
    Wait,
}
impl Command {
    fn to_str(&self) -> String {
        match self {
            Self::Buy(pos) => format!("{} {}", pos.y, pos.x),
            Self::Move((from, to)) => format!("{} {} {} {}", from.y, from.x, to.y, to.x),
            Self::Wait => String::from("-1"),
        }
    }
}

#[derive(Clone)]
struct State {
    day: usize,
    money: usize,
    machines: Vec<Coord>,
    machine_dim: Vec<Vec<bool>>,
    field: Vec<Vec<Option<Vegetable>>>,
    ans: Vec<Command>,
}
impl State {
    fn new() -> Self {
        State {
            day: 0,
            money: 1,
            machines: vec![],
            machine_dim: vec![vec![false; N]; N],
            field: vec![vec![None; N]; N],
            ans: vec![],
        }
    }

    fn buy_cost(&self) -> usize {
        let a = self.machines.len() + 1;
        a * a * a
    }

    // valid　な操作が来る前提
    fn action(&mut self, input: &Input, com: Command) {
        // println!("{}: {}", self.day, self.money);
        if self.day == T {
            return;
        }

        // do command
        match com {
            Command::Buy(pos) => {
                self.money -= self.buy_cost();
                self.machines.push(pos);
                pos.set_matrix(&mut self.machine_dim, true);
            }
            Command::Move((from, to)) => {
                self.machines.retain(|pos| *pos != from);
                self.machines.push(to);
                from.set_matrix(&mut self.machine_dim, false);
                to.set_matrix(&mut self.machine_dim, true);
            }
            Command::Wait => (),
        }
        self.ans.push(com);

        // put veget
        for i in 0..M {
            let veg = &input.vegets[i];
            if veg.s_day > self.day {
                break;
            }
            if veg.s_day == self.day {
                self.field[veg.pos.y as usize][veg.pos.x as usize] = Some(veg.clone());
            }
        }

        // calc money
        for machine in &self.machines {
            if let Some(veg) = machine.access_matrix(&self.field) {
                // TODO: machine が全て連結してる前提になっているが、ちゃんと計算する
                let mut dp = vec![vec![false; N]; N];
                let mut cnt = 1;
                let mut q = VecDeque::new();
                veg.pos.set_matrix(&mut dp, true);
                q.push_back(veg.pos.clone());
                while !q.is_empty() {
                    let pos = q.pop_front().unwrap();
                    for e in pos.mk_4dir() {
                        if !e.access_matrix(&dp) && *e.access_matrix(&self.machine_dim) {
                            cnt += 1;
                            e.set_matrix(&mut dp, true);
                            q.push_back(e);
                        }
                    }
                }

                self.money += veg.value * cnt;
                machine.set_matrix(&mut self.field, None);
            }
        }

        // delete veget
        for y in 0..N {
            for x in 0..N {
                if let Some(veg) = &self.field[y][x] {
                    if veg.e_day == self.day {
                        self.field[y][x] = None;
                    }
                }
            }
        }

        // day incr
        self.day += 1;
    }
}

#[fastout]
fn main() {
    let system_time = SystemTime::now();
    let mut _rng = thread_rng();

    input! {
        _: usize,
        _: usize,
        _: usize,
        rcsev: [(usize, usize, usize, usize, usize); M],
    }

    let input = Input::new(rcsev);
    let mut st = State::new();

    for d in 0..T {
        let n = st.machines.len();
        let command = if st.money >= st.buy_cost() && n < N + 13 {
            let pos = Coord::from_usize_pair((n % N, n / N));
            Command::Buy(pos)
        } else {
            Command::Wait
        };

        st.action(&input, command);
    }

    for com in st.ans.iter() {
        println!("{}", com.to_str());
    }

    eprintln!("score: {}", st.money);
    eprintln!("{}ms", system_time.elapsed().unwrap().as_millis());
}
