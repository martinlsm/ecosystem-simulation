use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
};

use bevy::prelude::*;

use crate::simulation::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GridPosition {
    pub column: usize,
    pub row: usize,
}

#[derive(Resource)]
pub struct NavigationGrid {
    blocked: [[bool; MAP_COLUMNS]; MAP_ROWS],
}

impl NavigationGrid {
    pub fn new() -> Self {
        Self {
            blocked: [[false; MAP_COLUMNS]; MAP_ROWS],
        }
    }

    pub fn set_blocked(&mut self, position: GridPosition, blocked: bool) {
        self.blocked[position.row][position.column] = blocked;
    }

    pub fn is_blocked(&self, position: Vec3) -> bool {
        let cell = Self::world_to_grid(position);
        self.blocked[cell.row][cell.column]
    }

    pub fn world_to_grid(position: Vec3) -> GridPosition {
        let column = ((position.x + MAP_WIDTH / 2.0) / TILE_WIDTH)
            .floor()
            .clamp(0.0, (MAP_COLUMNS - 1) as f32) as usize;
        let row = ((position.y + MAP_HEIGHT / 2.0) / TILE_HEIGHT)
            .floor()
            .clamp(0.0, (MAP_ROWS - 1) as f32) as usize;
        GridPosition { column, row }
    }

    pub fn grid_to_world(position: GridPosition) -> Vec3 {
        Vec3::new(
            -MAP_WIDTH / 2.0 + TILE_WIDTH * (position.column as f32 + 0.5),
            -MAP_HEIGHT / 2.0 + TILE_HEIGHT * (position.row as f32 + 0.5),
            0.0,
        )
    }

    pub fn find_path(
        &self,
        start: GridPosition,
        goal: GridPosition,
    ) -> Option<VecDeque<GridPosition>> {
        if self.blocked[start.row][start.column] || self.blocked[goal.row][goal.column] {
            return None;
        }

        let mut previous = [[None; MAP_COLUMNS]; MAP_ROWS];
        let mut costs = [[usize::MAX; MAP_COLUMNS]; MAP_ROWS];
        let mut open_set = BinaryHeap::new();
        costs[start.row][start.column] = 0;
        open_set.push((Reverse(Self::heuristic(start, goal)), Reverse(0), start));

        while let Some((Reverse(_), Reverse(current_cost), current)) = open_set.pop() {
            if current_cost != costs[current.row][current.column] {
                continue;
            }

            if current == goal {
                break;
            }

            for neighbor in self.neighbors(current) {
                if let Some(neighbor) = neighbor {
                    if self.blocked[neighbor.row][neighbor.column] {
                        continue;
                    }

                    let neighbor_cost = current_cost + 1;
                    if neighbor_cost < costs[neighbor.row][neighbor.column] {
                        costs[neighbor.row][neighbor.column] = neighbor_cost;
                        previous[neighbor.row][neighbor.column] = Some(current);
                        let priority = neighbor_cost + Self::heuristic(neighbor, goal);
                        open_set.push((Reverse(priority), Reverse(neighbor_cost), neighbor));
                    }
                }
            }
        }

        if costs[goal.row][goal.column] == usize::MAX {
            return None;
        }

        let mut path = VecDeque::new();
        let mut current = goal;
        while current != start {
            path.push_front(current);
            current = previous[current.row][current.column]?;
        }
        Some(path)
    }

    fn heuristic(from: GridPosition, to: GridPosition) -> usize {
        from.column.abs_diff(to.column) + from.row.abs_diff(to.row)
    }

    fn neighbors(&self, position: GridPosition) -> [Option<GridPosition>; 4] {
        [
            (position.column > 0).then_some(GridPosition {
                column: position.column - 1,
                row: position.row,
            }),
            (position.column + 1 < MAP_COLUMNS).then_some(GridPosition {
                column: position.column + 1,
                row: position.row,
            }),
            (position.row > 0).then_some(GridPosition {
                column: position.column,
                row: position.row - 1,
            }),
            (position.row + 1 < MAP_ROWS).then_some(GridPosition {
                column: position.column,
                row: position.row + 1,
            }),
        ]
    }
}
