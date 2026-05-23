fn main() {
    let mut total = 0u64;
    for seed in 0..50 {
        total = total.wrapping_add(run_workload(seed));
    }
    println!("result: {total}");
}
fn run_workload(seed: u64) -> u64 {
    let numbers = generate_numbers(seed, 2_000_000);
    let filtered = filter_numbers(&numbers);
    calculate_score(&filtered)
}
fn generate_numbers(seed: u64, count: usize) -> Vec<u64> {
    let mut numbers = Vec::with_capacity(count);
    let mut value = seed + 1;
    for _ in 0..count {
        value = pseudo_random(value);
        numbers.push(value);
    }
    numbers
}
fn pseudo_random(mut value: u64) -> u64 {
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    value
}
fn filter_numbers(numbers: &[u64]) -> Vec<u64> {
    numbers
        .iter()
        .copied()
        .filter(|n| n % 3 == 0 || n % 7 == 0)
        .collect()
}
fn calculate_score(numbers: &[u64]) -> u64 {
    let mut score = 0u64;
    for &number in numbers {
        score = score.wrapping_add(expensive_calculation(number));
    }
    score
}
fn expensive_calculation(mut value: u64) -> u64 {
    for _ in 0..100 {
        value = value.rotate_left(5);
        value = value.wrapping_mul(31);
        value ^= 0x9e3779b97f4a7c15;
    }
    value
}
