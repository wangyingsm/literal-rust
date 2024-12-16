use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    convert::Infallible,
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Empty,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    East,
    South,
    West,
    North,
}

impl From<u8> for Direction {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::East,
            1 => Self::South,
            2 => Self::West,
            3 => Self::North,
            _ => unreachable!("never happened"),
        }
    }
}

impl Direction {
    pub fn turn_clockwise(self) -> Self {
        Self::from(((self as u8) + 1) % 4)
    }
    pub fn trun_counter_clockwise(self) -> Self {
        Self::from(((self as u8) + 3) % 4)
    }
}

#[derive(Debug)]
pub struct Maze {
    rows: usize,
    cols: usize,
    tiles: Vec<Tile>,
}

impl FromStr for Maze {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s.lines().count();
        let cols = s.lines().nth(0).unwrap().as_bytes().len();
        let tiles = s
            .lines()
            .flat_map(|l| {
                l.as_bytes().iter().map(|b| match b {
                    b'#' => Tile::Wall,
                    b'.' => Tile::Empty,
                    b'S' => Tile::Start,
                    b'E' => Tile::End,
                    _ => unreachable!("never happened"),
                })
            })
            .collect();
        Ok(Self { rows, cols, tiles })
    }
}

type Index = usize;

impl Maze {
    fn as_index(&self, row: usize, col: usize) -> Index {
        self.cols * row + col
    }
    fn as_row_col(&self, index: Index) -> (usize, usize) {
        (index / self.cols, index % self.cols)
    }
    fn east(&self, row: usize, col: usize) -> Option<(Index, Tile)> {
        if col == self.cols - 1 {
            return None;
        }
        Some((
            self.as_index(row, col + 1),
            self.tiles[self.as_index(row, col + 1)],
        ))
    }
    fn south(&self, row: usize, col: usize) -> Option<(Index, Tile)> {
        if row == self.rows - 1 {
            return None;
        }
        Some((
            self.as_index(row + 1, col),
            self.tiles[self.as_index(row + 1, col)],
        ))
    }
    fn west(&self, row: usize, col: usize) -> Option<(Index, Tile)> {
        if col == 0 {
            return None;
        }
        Some((
            self.as_index(row, col - 1),
            self.tiles[self.as_index(row, col - 1)],
        ))
    }
    fn north(&self, row: usize, col: usize) -> Option<(Index, Tile)> {
        if row == 0 {
            return None;
        }
        Some((
            self.as_index(row - 1, col),
            self.tiles[self.as_index(row - 1, col)],
        ))
    }
    fn next_tile(&self, row: usize, col: usize, dir: Direction) -> Option<(Index, Tile)> {
        match dir {
            Direction::East => self.east(row, col),
            Direction::South => self.south(row, col),
            Direction::West => self.west(row, col),
            Direction::North => self.north(row, col),
        }
    }
    fn dijkstra_walk(&self, start: Index, end: Index) -> usize {
        let mut visited = HashSet::new();
        let mut scores = HashMap::new();
        let mut visit_next = VecDeque::new();
        visit_next.push_front((start, 0, Direction::East));
        scores.insert(start, 0);
        while let Some((index, score, dir)) = visit_next.pop_front() {
            // if visited.contains(&index) {
            // continue;
            // }
            if index == end {
                break;
            }
            if let Entry::Occupied(mut e) = scores.entry(index) {
                if score < *e.get() {
                    *e.get_mut() = score;
                }
            }

            for (d, points) in [
                (dir, 1),
                (dir.turn_clockwise(), 1001),
                (dir.trun_counter_clockwise(), 1001),
            ] {
                let (row, col) = self.as_row_col(index);
                if let Some((next_index, next_tile)) = self.next_tile(row, col, d) {
                    let s = scores.entry(next_index).or_insert(usize::MAX);
                    if next_tile == Tile::Wall || *s <= score {
                        //|| visited.contains(&next_index) {
                        continue;
                    }
                    let next_score = score + points;
                    match scores.entry(next_index) {
                        Entry::Occupied(mut e) => {
                            if next_score < *e.get() {
                                *e.get_mut() = next_score;
                            }
                            if d == dir {
                                visit_next.push_front((next_index, *e.get(), d));
                            } else {
                                visit_next.push_back((next_index, *e.get(), d));
                            }
                        }
                        Entry::Vacant(e) => {
                            e.insert(next_score);
                            if d == dir {
                                visit_next.push_front((next_index, next_score, d));
                            } else {
                                visit_next.push_back((next_index, next_score, d));
                            }
                        }
                    }
                }
            }
            visited.insert(index);
        }
        *scores.get(&end).unwrap()
    }

    pub fn solution_part1(&self) -> usize {
        let mut start = 0;
        let mut end = 0;
        for (i, t) in self.tiles.iter().enumerate() {
            match t {
                Tile::Start => start = i,
                Tile::End => end = i,
                _ => (),
            }
        }
        self.dijkstra_walk(start, end)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const INPUT1: &str = r#"###############
#.......#....E#
#.#.###.#.###.#
#.....#.#...#.#
#.###.#####.#.#
#.#.#.......#.#
#.#.#####.###.#
#...........#.#
###.#.#####.#.#
#...#.....#.#.#
#.#.#.###.#.#.#
#.....#...#.#.#
#.###.#.#.#.#.#
#S..#.....#...#
###############"#;
    const INPUT2: &str = r#"#################
#...#...#...#..E#
#.#.#.#.#.#.#.#.#
#.#.#.#...#...#.#
#.#.#.#.###.#.#.#
#...#.#.#.....#.#
#.#.#.#.#.#####.#
#.#...#.#.#.....#
#.#.#####.#.###.#
#.#.#.......#...#
#.#.###.#####.###
#.#.#...#.....#.#
#.#.#.#####.###.#
#.#.#.........#.#
#.#.#.#########.#
#S#.............#
#################"#;

    #[test]
    fn test_walk_maze_recursive() {
        let maze: Maze = INPUT1.parse().unwrap();
        assert_eq!(
            maze.dijkstra_walk(maze.as_index(13, 1), maze.as_index(1, 13)),
            7036
        );
        let maze: Maze = INPUT2.parse().unwrap();
        assert_eq!(
            maze.dijkstra_walk(maze.as_index(15, 1), maze.as_index(1, 15)),
            11048
        );
    }
}
