#[allow(unused)]
fn main() {
    let a = vec![0; 1e7 as usize];
    let b = a.iter().map(|e| e + 1).collect::<Vec<_>>();
    return;
}
