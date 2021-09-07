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

#[derive(Clone, PartialEq, Eq)]
struct Vegetable {
    pos: Coord,
    s_day: usize, // s_day の頭に生える
    e_day: usize, // e_day の最後に消える
    value: usize,
}

#[derive(Clone)]
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

#[derive(Clone, PartialEq, Eq)]
enum Command {
    Buy(Coord),
    Move(Coord, Coord),
    Wait,
}
impl Command {
    fn to_str(&self) -> String {
        match self {
            Self::Buy(pos) => format!("{} {}", pos.y, pos.x),
            Self::Move(from, to) => format!("{} {} {} {}", from.y, from.x, to.y, to.x),
            Self::Wait => String::from("-1"),
        }
    }
}

struct BeamSearch {
    input: Input,
    init_score: isize,
}
impl bs::BeamSearch for BeamSearch {
    type State = State;

    // １ターンに１機械任意に増やせるシミュレート
    fn transit(&self, st: &Self::State) -> Vec<Self::State> {
        let mut res = vec![];

        let mut stay_next_st = st.clone();
        stay_next_st.action(&self.input, Command::Wait);
        res.push(stay_next_st);
        for neb in st.neighber_empty_blocks(&st.machines[0]) {
            let mut next_st = st.clone();
            // お金を払わずにマシンをセットする(ことで任意に増やせる場合のシミュレートをする)
            next_st.money += st.buy_cost();
            next_st.action(&self.input, Command::Buy(neb));

            res.push(next_st.clone());
        }

        res
    }

    fn evaluate(&self, st: &Self::State) -> isize {
        st.money as isize - self.init_score
    }
}

// その日の野菜は置かれた状態で始める
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
    fn new(input: &Input) -> Self {
        let mut st = State {
            day: 0,
            money: 1,
            machines: vec![],
            machine_dim: vec![vec![false; N]; N],
            field: vec![vec![None; N]; N],
            ans: vec![],
        };
        st.put_veget(&input);
        st
    }

    fn buy_cost(&self) -> usize {
        let a = self.machines.len() + 1;
        a * a * a
    }
    // Buyコマンドを使えるか
    fn can_buy(&self) -> bool {
        self.buy_cost() <= self.money
    }

    fn set_machine(&mut self, pos: &Coord) {
        self.machines.push(pos.clone());
        pos.set_matrix(&mut self.machine_dim, true);
    }
    fn delete_machine(&mut self, pos: &Coord) {
        self.machines.retain(|p| *p != *pos);
        pos.set_matrix(&mut self.machine_dim, false);
    }

    // その日の残っているvalue
    fn get_today_value(&self, pos: &Coord) -> usize {
        pos.access_matrix(&self.field)
            .clone()
            .map(|veget| veget.value)
            .unwrap_or(0)
    }

    // その日が開始日の野菜の設置
    fn put_veget(&mut self, input: &Input) {
        let bs = BinarySearch { day: self.day };
        let m = bs.solve(0, M, &input.vegets);
        for i in m..M {
            let veg = &input.vegets[i];
            if veg.s_day > self.day {
                break;
            }
            if veg.s_day == self.day {
                self.field[veg.pos.y as usize][veg.pos.x as usize] = Some(veg.clone());
            }
        }
    }

    // posの機械群に隣接する空きマスの一覧を返す
    fn neighber_empty_blocks(&self, pos: &Coord) -> Vec<Coord> {
        let mut res = vec![];

        let mut dp = vec![vec![false; N]; N];
        let mut q = VecDeque::new();
        pos.set_matrix(&mut dp, true);
        q.push_back(pos.clone());
        while !q.is_empty() {
            let pos = q.pop_front().unwrap();
            for e in pos.mk_4dir() {
                if !e.access_matrix(&dp) {
                    if *e.access_matrix(&self.machine_dim) {
                        e.set_matrix(&mut dp, true);
                        q.push_back(e);
                    } else {
                        res.push(e);
                    }
                }
            }
        }

        res
    }

    // pos に設置された機械と連結してる個数を返す（自身も数える）
    fn count_connections(&self, pos: &Coord) -> usize {
        let mut dp = vec![vec![false; N]; N];
        let mut cnt = 1;
        let mut q = VecDeque::new();
        pos.set_matrix(&mut dp, true);
        q.push_back(pos.clone());
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

        cnt
    }

    fn can_cut_in_keep_connect(&mut self, pos: &Coord) -> bool {
        self.delete_machine(&pos);
        // 始点候補達
        let mut sps: Vec<Coord> = pos
            .mk_4dir()
            .into_iter()
            .filter(|p| *p.access_matrix(&self.machine_dim))
            .collect();
        let cnt = match sps.pop() {
            None => 0,
            Some(p) => self.count_connections(&p),
        };

        self.set_machine(&pos);

        cnt == self.machines.len() - 1
    }

    // valid　な操作が来る前提
    fn action(&mut self, input: &Input, com: Command) {
        if self.day == T {
            return;
        }

        // do command
        match com {
            Command::Buy(pos) => {
                self.money -= self.buy_cost();
                self.set_machine(&pos);
            }
            Command::Move(from, to) => {
                self.delete_machine(&from);
                self.set_machine(&to);
            }
            Command::Wait => (),
        }
        self.ans.push(com);

        // calc money
        for machine in &self.machines {
            if let Some(veg) = machine.access_matrix(&self.field) {
                self.money += veg.value * self.count_connections(&veg.pos);
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

        // put veget
        // 翌日分の野菜を設置して終える
        self.put_veget(input);
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
    let mut st = State::new(&input);

    let bs_opt = bs::BeamSearchOption {
        beam_width: 10,
        depth: 2,
    };

    for d in 0..T {
        if d % 100 == 0 {
            eprintln!("day: {}", d);
        }
        if d == 0 {
            let command = Command::Buy(Coord::from_usize_pair((N / 2, N / 2)));
            st.action(&input, command);
            continue;
        }

        // TODO: 毎回input cloneするの嫌そう
        let bs = BeamSearch {
            input: input.clone(),
            init_score: st.money as isize,
        };

        let ans_st = bs::search(&bs, st.clone(), &bs_opt);

        let n = (&st).machines.len();
        // Buyで持ってるやつが、置きたい箇所
        let to_target = ans_st.ans[d].clone();
        let command = match to_target {
            Command::Move(_, _) => unreachable!(),
            Command::Wait => Command::Wait,
            Command::Buy(pos) => {
                // 買えるなら買えばいい
                if st.can_buy() && n <= 35 {
                    Command::Buy(pos)
                } else {
                    let mut res = Command::Wait;

                    // 今ターン予定設置マスを妨げずに取り除る、一番減少スコアが小さいマスを探す(一手読み)
                    // TODO: 予定のもの全てを織り込みたい
                    let mut min_value = 1e15 as usize;
                    st.set_machine(&pos);
                    for machine in (&st).machines.clone().iter() {
                        if *machine == pos {
                            continue;
                        }
                        let value = st.get_today_value(&machine);
                        if value < min_value {
                            if st.can_cut_in_keep_connect(&machine) {
                                min_value = value;
                                res = Command::Move(machine.clone(), pos.clone());
                            }
                        }
                    }
                    st.delete_machine(&pos);

                    res
                }
            }
        };

        st.action(&input, command);
    }

    for com in st.ans.iter() {
        println!("{}", com.to_str());
    }

    eprintln!("score: {}", st.money);
    eprintln!("{}ms", system_time.elapsed().unwrap().as_millis());
}

#[allow(dead_code)]
mod bs {
    use std::cmp::Ordering;
    use std::collections::BinaryHeap;

    // 第一要素を比較対象とする組
    struct ForSort<T> {
        score: isize,
        node: T,
    }
    // ダミー
    impl<T> PartialEq for ForSort<T> {
        fn eq(&self, other: &Self) -> bool {
            self.score == other.score
        }
    }
    impl<T> Eq for ForSort<T> {}

    impl<T> PartialOrd for ForSort<T> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.score.partial_cmp(&other.score)
        }
    }
    impl<T> Ord for ForSort<T> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.score.cmp(&other.score)
        }
    }

    pub struct BeamSearchOption {
        pub beam_width: usize,
        pub depth: usize,
    }
    pub trait BeamSearch {
        type State: Clone;

        fn transit(&self, st: &Self::State) -> Vec<Self::State>;
        fn evaluate(&self, st: &Self::State) -> isize;
    }

    pub fn search<A: BeamSearch>(bs: &A, init_st: A::State, opt: &BeamSearchOption) -> A::State {
        let mut pq: BinaryHeap<ForSort<A::State>> = BinaryHeap::new();
        pq.push(ForSort {
            score: bs.evaluate(&init_st),
            node: init_st.clone(),
        });
        for _ in 1..=opt.depth {
            let mut next_pq: BinaryHeap<ForSort<A::State>> = BinaryHeap::new();
            for _ in 0..opt.beam_width {
                if pq.is_empty() {
                    break;
                } else {
                    let st = pq.pop().unwrap().node;
                    for next_st in bs.transit(&st) {
                        next_pq.push(ForSort {
                            score: bs.evaluate(&next_st),
                            node: next_st,
                        })
                    }
                }
            }
            pq = next_pq;
        }
        pq.pop().unwrap().node
    }
}

// 条件を満たす最小の値を返す
struct BinarySearch {
    day: usize,
}

impl BinarySearch {
    fn check(&self, i: usize, vegets: &Vec<Vegetable>) -> bool {
        vegets[i].s_day >= self.day
    }

    fn solve(&self, min: usize, max: usize, vegets: &Vec<Vegetable>) -> usize {
        let mid: usize = (max + min) / 2;
        match max - min {
            0 | 1 => match self.check(min, &vegets) {
                true => min,
                false => max,
            },
            _ => match self.check(mid, &vegets) {
                true => self.solve(min, mid, &vegets),
                false => self.solve(mid, max, &vegets),
            },
        }
    }
}
