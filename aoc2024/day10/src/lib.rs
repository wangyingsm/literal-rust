use std::{collections::HashSet, convert::Infallible, str::FromStr};

#[derive(Debug)]
pub struct HikingMap {
    cols: usize,
    spots: Vec<u8>,
}

impl FromStr for HikingMap {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cols = s.lines().nth(0).unwrap().as_bytes().len();
        let spots = s
            .lines()
            .flat_map(|l| {
                l.as_bytes().iter().map(|b| match b {
                    b @ b'0'..=b'9' => b - b'0',
                    _ => panic!("never happened"),
                })
            })
            .collect();
        Ok(Self { cols, spots })
    }
}

type Index = usize;

impl HikingMap {
    fn as_index(&self, row: usize, col: usize) -> Index {
        row * self.cols + col
    }

    fn as_row_col(&self, index: Index) -> (usize, usize) {
        (index / self.cols, index % self.cols)
    }

    fn left(&self, row: usize, col: usize) -> Option<u8> {
        if col == 0 {
            return None;
        }
        Some(self.spots[self.as_index(row, col - 1)])
    }

    fn right(&self, row: usize, col: usize) -> Option<u8> {
        if col == self.cols - 1 {
            return None;
        }
        Some(self.spots[self.as_index(row, col + 1)])
    }

    fn up(&self, row: usize, col: usize) -> Option<u8> {
        if row == 0 {
            return None;
        }
        Some(self.spots[self.as_index(row - 1, col)])
    }

    fn down(&self, row: usize, col: usize) -> Option<u8> {
        let rows = self.spots.len() / self.cols;
        if row == rows - 1 {
            return None;
        }
        Some(self.spots[self.as_index(row + 1, col)])
    }

    fn find_trail_recursive(&self, row: usize, col: usize, distinct_nines: &mut HashSet<Index>) {
        let index = self.as_index(row, col);
        let height = self.spots[index];
        if height == 9 {
            distinct_nines.insert(index);
            return;
        }
        if let Some(left_height) = self.left(row, col) {
            if left_height == height + 1 {
                self.find_trail_recursive(row, col - 1, distinct_nines);
            }
        }
        if let Some(right_height) = self.right(row, col) {
            if right_height == height + 1 {
                self.find_trail_recursive(row, col + 1, distinct_nines);
            }
        }
        if let Some(up_height) = self.up(row, col) {
            if up_height == height + 1 {
                self.find_trail_recursive(row - 1, col, distinct_nines);
            }
        }
        if let Some(down_height) = self.down(row, col) {
            if down_height == height + 1 {
                self.find_trail_recursive(row + 1, col, distinct_nines);
            }
        }
    }

    pub fn solution_part1(&self) -> usize {
        self.spots
            .iter()
            .enumerate()
            .filter(|(_, x)| **x == 0)
            .map(|(index, _)| {
                let (row, col) = self.as_row_col(index);
                let mut distinct_nines = HashSet::new();
                self.find_trail_recursive(row, col, &mut distinct_nines);
                distinct_nines.len()
            })
            .sum()
    }

    fn find_trail_distinct_recursive(&self, row: usize, col: usize) -> usize {
        let index = self.as_index(row, col);
        let height = self.spots[index];
        if height == 9 {
            return 1;
        }
        let mut trails_count = 0;
        if let Some(left_height) = self.left(row, col) {
            if left_height == height + 1 {
                trails_count += self.find_trail_distinct_recursive(row, col - 1);
            }
        }
        if let Some(right_height) = self.right(row, col) {
            if right_height == height + 1 {
                trails_count += self.find_trail_distinct_recursive(row, col + 1);
            }
        }
        if let Some(up_height) = self.up(row, col) {
            if up_height == height + 1 {
                trails_count += self.find_trail_distinct_recursive(row - 1, col);
            }
        }
        if let Some(down_height) = self.down(row, col) {
            if down_height == height + 1 {
                trails_count += self.find_trail_distinct_recursive(row + 1, col);
            }
        }
        trails_count
    }

    pub fn solution_part2(&self) -> usize {
        self.spots
            .iter()
            .enumerate()
            .filter_map(|(index, x)| {
                if *x == 0 {
                    let (row, col) = self.as_row_col(index);
                    Some(self.find_trail_distinct_recursive(row, col))
                } else {
                    None
                }
            })
            .sum()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const INPUT: &str = r#"89010123
78121874
87430965
96549874
45678903
32019012
01329801
10456732"#;
    #[test]
    fn test_parse_from_str() {
        let hiking_map: HikingMap = INPUT.parse().unwrap();
        assert_eq!(hiking_map.cols, 8);
        assert_eq!(hiking_map.spots[0], 8);
        assert_eq!(hiking_map.spots[7], 3);
        assert_eq!(hiking_map.spots[27], 4);
        assert_eq!(hiking_map.spots[63], 2);
    }

    #[test]
    fn test_directions() {
        let hiking_map: HikingMap = INPUT.parse().unwrap();
        assert_eq!(hiking_map.left(0, 1), Some(8));
        assert_eq!(hiking_map.right(0, 6), Some(3));
        assert_eq!(hiking_map.up(4, 3), Some(4));
        assert_eq!(hiking_map.down(6, 7), Some(2));
        assert_eq!(hiking_map.right(5, 7), None);
    }

    #[test]
    fn test_hiking_trail_recursive() {
        let hiking_map: HikingMap = INPUT.parse().unwrap();
        let mut nines = HashSet::new();
        hiking_map.find_trail_recursive(0, 2, &mut nines);
        assert_eq!(nines.len(), 5);
        nines.clear();
        hiking_map.find_trail_recursive(0, 4, &mut nines);
        assert_eq!(nines.len(), 6);
        nines.clear();
        hiking_map.find_trail_recursive(2, 4, &mut nines);
        assert_eq!(nines.len(), 5);
        nines.clear();
        hiking_map.find_trail_recursive(4, 6, &mut nines);
        assert_eq!(nines.len(), 3);
        nines.clear();
        hiking_map.find_trail_recursive(5, 2, &mut nines);
        assert_eq!(nines.len(), 1);
        nines.clear();
        hiking_map.find_trail_recursive(5, 5, &mut nines);
        assert_eq!(nines.len(), 3);
        nines.clear();
        hiking_map.find_trail_recursive(6, 0, &mut nines);
        assert_eq!(nines.len(), 5);
        nines.clear();
        hiking_map.find_trail_recursive(6, 6, &mut nines);
        assert_eq!(nines.len(), 3);
        nines.clear();
        hiking_map.find_trail_recursive(7, 1, &mut nines);
        assert_eq!(nines.len(), 5);
    }

    #[test]
    fn test_hiking_trail_distinct_recursive() {
        let hiking_map: HikingMap = INPUT.parse().unwrap();
        let trails = hiking_map.find_trail_distinct_recursive(0, 2);
        assert_eq!(trails, 20);
        let trails = hiking_map.find_trail_distinct_recursive(0, 4);
        assert_eq!(trails, 24);
        let trails = hiking_map.find_trail_distinct_recursive(2, 4);
        assert_eq!(trails, 10);
        let trails = hiking_map.find_trail_distinct_recursive(4, 6);
        assert_eq!(trails, 4);
        let trails = hiking_map.find_trail_distinct_recursive(5, 2);
        assert_eq!(trails, 1);
        let trails = hiking_map.find_trail_distinct_recursive(5, 5);
        assert_eq!(trails, 4);
        let trails = hiking_map.find_trail_distinct_recursive(6, 0);
        assert_eq!(trails, 5);
        let trails = hiking_map.find_trail_distinct_recursive(6, 6);
        assert_eq!(trails, 8);
        let trails = hiking_map.find_trail_distinct_recursive(7, 1);
        assert_eq!(trails, 5);
    }
}
