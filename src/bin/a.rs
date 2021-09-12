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

const BEAM_WIDTH: usize = 4;
// 野菜の価値が最大価値の 1/VEGET_PRUNE_DIV を下回るケースを枝刈る
const VEGET_PRUNE_DIV: usize = 15;
const PUT_VEGET_AHEAD_DAY: usize = 10;
const PROSPECT_GAIN_WEIGHT: f64 = 0.5;
const MODE_CHANGE_DAY: usize = 400;
const BUY_END_DAY: usize = 825;

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

#[derive(Clone, PartialEq, Eq, Debug)]
struct Vegetable {
    pos: Coord,
    s_day: usize, // s_day の頭に生える
    e_day: usize, // e_day の最後に消える
    value: usize,
}
impl Vegetable {
    fn to_miniveget(&self) -> MiniVeget {
        MiniVeget {
            s_day: self.s_day,
            e_day: self.e_day,
            value: self.value,
        }
    }

    // その開始日で出せる最大値を返す
    fn day_max_value(&self) -> usize {
        2.0_f64.powf(1.0 + (self.s_day as f64 / 100.0)) as usize
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
struct MiniVeget {
    s_day: usize,
    e_day: usize,
    value: usize,
}
impl MiniVeget {
    fn to_veg(&self, pos: Coord) -> Vegetable {
        Vegetable {
            pos,
            s_day: self.s_day,
            e_day: self.e_day,
            value: self.value,
        }
    }
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

struct DpTable {
    table: Vec<Vec<usize>>,
    base: usize,
}
impl DpTable {
    fn new() -> Self {
        let base = 0;
        DpTable {
            table: vec![vec![base; N]; N],
            base,
        }
    }

    fn reset_base(&mut self) {
        self.base += 1;
    }

    fn check(&self, pos: &Coord) -> bool {
        *pos.access_matrix(&self.table) == self.base
    }

    fn done(&mut self, pos: &Coord) {
        pos.set_matrix(&mut self.table, self.base);
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
    dp_table: DpTable,
}
impl BeamSearchTrait for BeamSearch {
    type State = State;

    // １ターンに１機械任意に増やせるシミュレート
    fn transit(
        &mut self,
        st: &Self::State,
        rng: &mut ThreadRng,
        next_commands_vec_index: usize,
    ) -> Vec<Self::State> {
        let mut res = vec![];

        // 何もしないケース
        let mut stay_next_st = st.clone();
        stay_next_st.action(&self.input, Command::Wait, next_commands_vec_index);
        res.push(stay_next_st);

        // TODO: 中距離の高価値野菜が取れない
        let mut machines: Vec<Coord> = st.get_machines();
        machines.shuffle(rng);

        if machines.len() > 0 {
            let target_block = st
                .neighber_empty_blocks(&machines[0], &mut self.dp_table)
                .into_iter()
                .filter(|machine| {
                    // マシンが無くて、野菜があるマスに隣接している
                    machine.access_matrix(&st.field).is_some() || {
                        machine.mk_4dir().iter().any(|mov| {
                            !mov.access_matrix(&st.field).is_some() && st.machine_dim.get(&mov)
                        })
                    }
                });
            for neb in target_block {
                let mut next_st = st.clone();

                // 買えるなら買えばいい
                let command = next_st.make_command(neb, &machines, &mut self.dp_table);

                next_st.action(&self.input, command, next_commands_vec_index);

                res.push(next_st.clone());
            }
        }

        res
    }

    fn evaluate(&self, st: &Self::State) -> f64 {
        st.total_money as f64 + st.prospect_gain as f64 * PROSPECT_GAIN_WEIGHT
    }
}

// (y: 0~7, y:8~15). 16*16 == 256 == 128 * 2
// Vec<Vec<bool>> を表現. 横書き方向に１桁目から埋めていく.
#[derive(Clone)]
struct BoolMat(u128, u128);
impl BoolMat {
    fn get(&self, pos: &Coord) -> bool {
        if pos.y <= 7 {
            let i = pos.y * 16 + pos.x % 16;
            self.0 & (1 << i) > 0
        } else {
            let i = (pos.y - 8) * 16 + pos.x % 16;
            self.1 & (1 << i) > 0
        }
    }

    fn put(&mut self, pos: &Coord) {
        if pos.y <= 7 {
            let i = pos.y * 16 + pos.x % 16;
            self.0 = self.0 | (1 << i);
        } else {
            let i = (pos.y - 8) * 16 + pos.x % 16;
            self.1 = self.1 | (1 << i);
        }
    }

    fn delete(&mut self, pos: &Coord) {
        if pos.y <= 7 {
            let i = pos.y * 16 + pos.x % 16;
            if self.0 & (1 << i) > 0 {
                self.0 -= 1 << i;
            }
        } else {
            let i = (pos.y - 8) * 16 + pos.x % 16;
            if self.1 & (1 << i) > 0 {
                self.1 -= 1 << i;
            }
        }
    }
}

// その日の野菜は置かれた状態で始める
#[derive(Clone)]
struct State {
    day: usize,
    money: usize,
    total_money: usize, // これまでに得たお金の通算
    prospect_gain: usize,
    machines_num: usize,
    machine_dim: BoolMat,
    field: Vec<Vec<Option<MiniVeget>>>,
    this_turn_command: Command,
    pre_commands_index: usize, // 高速化目的. bs::search内で使う
}
impl State {
    fn new(input: &Input) -> Self {
        let mut st = State {
            day: 0,
            money: 1,
            total_money: 1,
            prospect_gain: 0,
            machines_num: 0,
            machine_dim: BoolMat(0, 0),
            field: vec![vec![None; N]; N],
            this_turn_command: Command::Wait,
            pre_commands_index: 0, // dummy
        };
        st.put_veget(&input);
        st
    }

    fn buy_cost(&self) -> usize {
        let a = self.machines_num + 1;
        a * a * a
    }
    // Buyコマンドを使えるか
    fn can_buy(&self) -> bool {
        self.buy_cost() <= self.money
    }

    fn set_machine(&mut self, pos: &Coord) {
        self.machines_num += 1;
        self.machine_dim.put(&pos);
    }
    fn delete_machine(&mut self, pos: &Coord) {
        self.machines_num -= 1;
        self.machine_dim.delete(&pos);
    }
    fn get_machines(&self) -> Vec<Coord> {
        let mut res = vec![];
        for y in 0..N {
            for x in 0..N {
                let pos = Coord::from_usize_pair((x, y));
                if self.machine_dim.get(&pos) {
                    res.push(pos);
                }
            }
        }
        res
    }

    // その日の残っているvalue
    fn get_today_value(&self, pos: &Coord) -> usize {
        pos.access_matrix(&self.field)
            .clone()
            .map(|veget| veget.value)
            .unwrap_or(0)
    }

    fn get_vegets(&mut self) -> Vec<Vegetable> {
        let mut res = vec![];
        for y in 0..N {
            for x in 0..N {
                let pos = Coord::from_usize_pair((x, y));
                if let Some(mini_veg) = pos.access_matrix(&self.field) {
                    let veg = mini_veg.to_veg(pos);
                    res.push(veg);
                }
            }
        }
        res
    }

    fn make_command(
        &mut self,
        put_pos: Coord,
        machines: &Vec<Coord>,
        dp_table: &mut DpTable,
    ) -> Command {
        if self.can_buy() && self.day <= BUY_END_DAY {
            Command::Buy(put_pos)
        } else {
            let mut res = Command::Wait;

            // 今ターン予定設置マスを妨げずに取り除ける、一番減少スコアが小さいマスを探す(一手読み)
            // TODO: 予定のもの全てを織り込みたい
            let mut min_value = 1e15 as usize;
            self.set_machine(&put_pos);
            for machine in machines.iter() {
                if *machine == put_pos {
                    continue;
                }
                let value = self.get_today_value(&machine);
                if value < min_value {
                    if self.can_cut_in_keep_connect(&machine, dp_table) {
                        min_value = value;
                        res = Command::Move(machine.clone(), put_pos.clone());
                    }
                }
            }
            self.delete_machine(&put_pos);

            res
        }
    }

    // その日が開始日の野菜の設置
    fn put_veget(&mut self, input: &Input) {
        let bs = BinarySearch { day: self.day };
        let m = bs.solve(0, input.vegets.len() - 1, &input.vegets);
        for i in m..input.vegets.len() {
            let veg = &input.vegets[i];
            if veg.s_day > self.day + PUT_VEGET_AHEAD_DAY {
                break;
            }
            if veg.s_day <= self.day + PUT_VEGET_AHEAD_DAY {
                if *veg.pos.access_matrix(&self.field) == None {
                    self.field[veg.pos.y as usize][veg.pos.x as usize] = Some(veg.to_miniveget());
                }
            }
        }
    }

    // posの機械群に隣接する空きマスの一覧を返す
    fn neighber_empty_blocks(&self, pos: &Coord, dp_table: &mut DpTable) -> Vec<Coord> {
        let mut res = vec![];

        dp_table.reset_base();
        let mut q = VecDeque::new();
        dp_table.done(&pos);
        q.push_back(pos.clone());
        while !q.is_empty() {
            let pos = q.pop_front().unwrap();
            for e in pos.mk_4dir() {
                if !dp_table.check(&e) {
                    if self.machine_dim.get(&e) {
                        dp_table.done(&e);
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
    fn count_connections(&self, pos: &Coord, dp_table: &mut DpTable) -> usize {
        dp_table.reset_base();
        let mut cnt = 1;
        let mut q = VecDeque::new();
        dp_table.done(&pos);
        q.push_back(pos.clone());
        while !q.is_empty() {
            let pos = q.pop_front().unwrap();
            for e in pos.mk_4dir() {
                if !dp_table.check(&e) && self.machine_dim.get(&e) {
                    cnt += 1;
                    dp_table.done(&e);
                    q.push_back(e);
                }
            }
        }

        cnt
    }

    fn can_cut_in_keep_connect(&mut self, pos: &Coord, dp_table: &mut DpTable) -> bool {
        self.delete_machine(&pos);
        // 始点候補達
        let mut sps: Vec<Coord> = pos
            .mk_4dir()
            .into_iter()
            .filter(|p| self.machine_dim.get(&p))
            .collect();
        let cnt = match sps.pop() {
            None => 0,
            Some(p) => self.count_connections(&p, dp_table),
        };

        self.set_machine(&pos);

        cnt == self.machines_num - 1
    }

    // valid　な操作が来る前提
    fn action(&mut self, input: &Input, com: Command, next_commands_vec_index: usize) {
        if self.day == T {
            return;
        }

        // 高速化用。search関数内で使う。
        if self.day < MODE_CHANGE_DAY {
            self.pre_commands_index = next_commands_vec_index;
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
        self.this_turn_command = com;

        // calc money
        // Vecのメモリ割り当てを避けるために、get_machines を使っていない
        for y in 0..N {
            for x in 0..N {
                let pos = Coord::from_usize_pair((x, y));
                if self.machine_dim.get(&pos) {
                    let machine = pos;
                    if let Some(veg) = machine.access_matrix(&self.field) {
                        if self.day >= veg.s_day {
                            // TODO: 連結状態は維持すると決めているため、固定値埋め込みにしているが、後々これで行けるかはわからん
                            // let gain = veg.value * self.count_connections(&veg.pos);
                            let gain = veg.value * self.machines_num;
                            self.money += gain;
                            self.total_money += gain;
                            machine.set_matrix(&mut self.field, None);
                        } else {
                            // 未来に設置される野菜分も加点する
                            // TODO: ある程度減点しても良いかも

                            // TODO: 連結状態は維持すると決めているため、固定値埋め込みにしているが、後々これで行けるかはわからん
                            // let gain = veg.value * self.count_connections(&veg.pos);
                            self.prospect_gain = veg.value * self.machines_num;
                        }
                    }
                }
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
    let mut rng = thread_rng();

    input! {
        _: usize,
        _: usize,
        _: usize,
        rcsev: [(usize, usize, usize, usize, usize); M],
    }

    let mut input = Input::new(rcsev);
    // TODO: 野菜の価値が最大価値の5%を下回るケースを刈っている
    // input0で3,626個になった。
    input.vegets = input
        .vegets
        .into_iter()
        .filter(|veg| veg.day_max_value() / VEGET_PRUNE_DIV < veg.value)
        .collect::<Vec<_>>();
    eprintln!("{}", input.vegets.len());
    let mut st = State::new(&input);

    // 初日
    let command = Command::Buy(Coord::from_usize_pair((N / 2, N / 2)));
    st.action(&input, command, 0);

    // 二日目以降
    let bs_opt = BeamSearchOption {
        beam_width: BEAM_WIDTH,
        depth: MODE_CHANGE_DAY - 1,
    };
    let mut bs = BeamSearch {
        input: input.clone(),
        dp_table: DpTable::new(),
    };

    let (mid_ans, mid_st) = search(&mut bs, st, &bs_opt, &mut rng);
    for com in mid_ans.iter() {
        println!("{}", com.to_str());
    }

    // 800日以降
    let mut st = mid_st;
    let mut dp_table = DpTable::new();
    while st.day < T {
        // eprintln!("{}", st.day);
        // TODO: 残り数ターンは上位のマスの内 価値 / 距離 が最も大きいものに向かうようにしたい
        let vegs = st.get_vegets();

        // (価値/距離) が大きい順に並んだvegのリスト
        // (score, dist, veg) vec
        let machines = st.get_machines();
        let mut v: Vec<_> = vegs
            .iter()
            .map(|veg| {
                let dist = machines
                    .iter()
                    .map(|machine| machine.distance(&veg.pos))
                    .min()
                    .unwrap();
                (veg.value as f64 / dist as f64, dist, veg)
            })
            .filter(|(_, dist, _)| *dist > 0)
            .collect();
        v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        let mut flag = false;
        for (_, dist, veg) in v {
            if min(veg.e_day - st.day, T - st.day) >= dist as usize {
                // 辿り着くまでのコマンドを出力&stに適用
                dp_table.reset_base();
                let mut q = VecDeque::new();
                dp_table.done(&veg.pos);
                q.push_back((veg.pos.clone(), vec![veg.pos.clone()]));
                while !q.is_empty() {
                    let (pos, mut coms) = q.pop_front().unwrap();
                    // machine群にくっついた
                    if st.machine_dim.get(&pos) {
                        // 辿り着くまでのコマンドを出力&stに適用
                        // eprintln!("{:?} {}", coms, dist);
                        coms.pop();
                        for &p in coms.iter().rev() {
                            let command = st.make_command(p, &st.get_machines(), &mut dp_table);
                            println!("{}", command.to_str());
                            st.action(&input, command, 42);
                        }
                        //
                        flag = true;
                        break;
                    } else {
                        for next_pos in pos.mk_4dir() {
                            if !dp_table.check(&next_pos) {
                                dp_table.done(&next_pos);
                                let mut next_coms = coms.clone();
                                next_coms.push(next_pos);
                                q.push_back((next_pos, next_coms))
                            }
                        }
                    }
                }
            }
            if flag {
                break;
            }
        }

        if !flag {
            st.action(&input, Command::Wait, 0);
            println!("{}", Command::Wait.to_str());
        }
    }

    eprintln!("{}ms", system_time.elapsed().unwrap().as_millis());
}

use std::cmp::Ordering;
use std::collections::BinaryHeap;

// 第一要素を比較対象とする組
pub struct ForSort<T> {
    score: f64,
    pub node: T,
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
        // TODO: partial_cmp で大丈夫か
        self.score.partial_cmp(&other.score).unwrap()
    }
}

pub struct BeamSearchOption {
    pub beam_width: usize,
    pub depth: usize,
}
pub trait BeamSearchTrait {
    type State: Clone;

    fn transit(
        &mut self,
        st: &Self::State,
        rng: &mut ThreadRng,
        next_commands_vec_index: usize,
    ) -> Vec<Self::State>;
    fn evaluate(&self, st: &Self::State) -> f64;
}

fn search(
    bs: &mut BeamSearch,
    init_st: State,
    opt: &BeamSearchOption,
    rng: &mut ThreadRng,
) -> (Vec<Command>, State) {
    let mut pq: BinaryHeap<ForSort<State>> = BinaryHeap::new();
    let mut pre_commands_vec: Vec<Vec<Command>> = vec![vec![]];
    pq.push(ForSort {
        score: bs.evaluate(&init_st),
        node: init_st.clone(),
    });
    for d in 1..=opt.depth {
        let mut next_pq: BinaryHeap<ForSort<State>> = BinaryHeap::new();
        let mut next_commands_vec = vec![];
        if d % 100 == 0 {
            // eprintln!("day: {}", d);
        }
        for _ in 0..opt.beam_width {
            if pq.is_empty() {
                break;
            } else {
                let st = pq.pop().unwrap().node;
                // if d == 4 {
                //     eprintln!("{} {}", st.total_money, st.prospect_gain);
                // }
                let mut commands = pre_commands_vec[st.pre_commands_index].clone();
                commands.push(st.this_turn_command.clone());

                next_commands_vec.push(commands);
                let ncvi = next_commands_vec.len() - 1;
                for next_st in bs.transit(&st, rng, ncvi) {
                    next_pq.push(ForSort {
                        score: bs.evaluate(&next_st),
                        node: next_st,
                    })
                }
            }
        }
        pq = next_pq;
        pre_commands_vec = next_commands_vec;
    }
    let ans_st = pq.pop().unwrap().node;
    eprintln!("score: {}", ans_st.money);
    let mut commands = pre_commands_vec[ans_st.pre_commands_index].clone();
    commands.push(ans_st.this_turn_command.clone());

    (commands, ans_st)
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
