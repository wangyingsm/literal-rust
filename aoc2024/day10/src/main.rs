use day10::HikingMap;

fn main() {
    let input = include_str!("../input.txt");
    let hiking_map: HikingMap = input.parse().unwrap();
    println!("Day 10 Part 1: {}", hiking_map.solution_part1());
    println!("Day 10 Part 2: {}", hiking_map.solution_part2());
}
