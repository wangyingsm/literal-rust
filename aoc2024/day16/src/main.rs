use day16::Maze;

fn main() {
    let maze: Maze = include_str!("../input.txt").parse().unwrap();
    println!("Day 16 Part 1: {}", maze.solution_part1());
}
