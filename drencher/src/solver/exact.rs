//! Exact Solver
//!
//! This solver always finds an optimal solution (with as few moves as
//! possible). Currently, this solver is very slow and can't really handle
//! instances of size 8 and above.
//!
//! ## Algorithm
//!
//! The idea is simple: the solver traverses the whole game tree in order to
//! find a solution. Traversing is done with breadth-first-search which means
//! that the first solution we find is optimal. The theoretical game tree has
//! 'm^c' nodes, where 'm' is the number of moves of one optimal solution and
//! 'c' is the number of colors. One can easily imagine that trees of bigger
//! instances are impossible to search through. Therefore we have to implement
//! some techniques to avoid searching the *whole* tree.
//!
//! Currently only one optimization is used: when branching deeper into the
//! tree, we ensure that the color of the move corresponding to the current
//! branch, is even a neighbor of the field of cells. If not, the move is
//! useless and the subtree can be ignored. Obviously.
//!
//! ## Representing the tree
//!
//! Right now, the tree is not really represented as a tree. Since we're
//! progressing the tree in BFS, we only need to save the last/current layer.
//! Each node in this layer is saved as a `State` which saves a board state and
//! all moves leading to this state.
//!
//!
//!
use board::Board;
use color::Color;
use super::{Solver, Solution};
use std::collections::HashMap;
use std::iter::repeat;
use std::ops;

/// Type definition of exact solver. See module documentation for more
/// information.
pub struct Exact;

type Pos = (u8, u8);

/// Used to represent one node in the game tree. See module documentation for
/// more information.
#[derive(Clone)]
struct State {
    pub moves: Vec<Color>,
    pub board: Board,
}

impl Solver for Exact {
    fn solve(&self, _b: Board) -> Result<Solution, Solution> {
        // TODO: this is just debugging
        let b = Board::deterministic_random(3, 2);
        println!("{}", b);

        println!("{:#?}", generate_graph(&b));

        Ok(vec![])
    }
}

/// Generates the initial undirected graph representing the board. Every island
/// of multiple cells of the same color are represented by one node. Each node
/// contains the id's of all it's neighbors.
fn generate_graph(b: &Board) -> Graph {
    // We map every cell to the corresponding node to check if we already
    // processed that cell
    let mut g = Graph::default();
    let mut map = HashMap::new();

    // It doesn't matter in which order we progress the cells
    for x in 0..b.size() {
        for y in 0..b.size() {
            // If we already created a node for this cell, we skip it
            if map.contains_key(&(x, y)) {
                continue;
            }

            // We need to get all cells of the current island as well as the
            // adjacent cells. Note that these are only the directly adjacent
            // cells!
            let (island, adjacent) = get_island(b, (x, y));

            debug!(
                "({}, {}) {} => {:?} || {:?}",
                x, y, b[(x, y)], island, adjacent
            );

            // Add a new node with the color of the current cell and create an
            // alias for the index of the inserted node.
            let new_id = g.nodes.len();
            g.nodes.push(Node {
                adjacent: vec![],
                color: b[(x, y)],
            });

            // We need to add all cells of the current island to the cell-node
            // map to mark them progressed.
            for pos in island {
                map.insert(pos, new_id);
            }

            // Here we add the edges. Note that we only add edges to nodes
            // that already exist. This is perfectly fine because:
            // - we add an edge to every existing node now
            // - every node that will be created afterwards will add exactly
            //   one edge from that node to the current node
            for pos in adjacent {
                if let Some(&id) = map.get(&pos) {
                    g.nodes[id].adjacent.push(new_id);
                    g.nodes[new_id].adjacent.push(id);
                }
            }
        }
    }

    g
}

/// Returns all cells of the island around the given position as well as the
/// adjacent cells. Note that it only returns directly adjacent cells! This
/// means that if a bigger island is adjacent, only the cells directly touching
/// "our" island are returned. This is fine for our use-case.
fn get_island(b: &Board, (x, y): Pos) -> (Vec<Pos>, Vec<Pos>) {
    // Starting from the given position, we successively add neighbors to the
    // queue, which we haven't added yet. This is a depth first search in the
    // current implementation, but the order of visits doesn't matter.
    let mut to_visit = vec![(x, y)];
    let mut visited = CellMap::new(b.size(), false);
    visited[(x, y)] = true;

    // Vec's to collect the result.
    let mut island = Vec::new();
    let mut adjacent = Vec::new();

    // Alias for the size and the color of the initial position.
    let size = b.size();
    let init_color = b[(x, y)];

    // As long as we still have to visit a cell ...
    while let Some((x, y)) = to_visit.pop() {
        // If the current cell belongs to our island ...
        if b[(x, y)] == init_color {
            // Add to result vector
            island.push((x, y));

            // Add all neighbors that we haven't visited yet
            macro_rules! add_neighbor {
                ($pos:expr, $cond:expr) => {
                    if $cond && !visited[$pos] {
                        to_visit.push($pos);
                        visited[$pos] = true;
                    }
                }
            }

            add_neighbor!((x - 1, y), x > 0);
            add_neighbor!((x + 1, y), x < size - 1);
            add_neighbor!((x, y - 1), y > 0);
            add_neighbor!((x, y + 1), y < size - 1);
        } else {
            // ... otherwise it was added by a cell in the island, hence it's
            // a directly adjacent cell.
            adjacent.push((x, y));
        }
    }

    (island, adjacent)
}

#[derive(Default, Clone, Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub adjacent: Vec<usize>,
    pub color: Color,
}

struct CellMap<T> {
    size: u8,
    cells: Vec<T>,
}

impl<T> CellMap<T> {
    pub fn new(size: u8, obj: T) -> Self
        where T: Clone
    {
        CellMap {
            size: size,
            cells: repeat(obj).take((size as usize).pow(2)).collect(),
        }
    }

    pub fn default(size: u8) -> Self
        where T: Default
    {
        let cells = repeat(())
            .map(|_| T::default())
            .take((size as usize).pow(2))
            .collect();
        CellMap {
            size: size,
            cells: cells,
        }
    }
}


impl<T> ops::Index<(u8, u8)> for CellMap<T> {
    type Output = T;
    fn index(&self, (x, y): (u8, u8)) -> &Self::Output {
        if x > self.size || y > self.size {
            panic!(
                "x ({}) or y ({}) greater than size ({})",
                x, y, self.size
            );
        }

        &self.cells[
            (y as usize) * (self.size as usize)
                + (x as usize)
        ]
    }
}

impl<T> ops::IndexMut<(u8, u8)> for CellMap<T> {
    fn index_mut(&mut self, (x, y): (u8, u8)) -> &mut Self::Output {
        if x > self.size || y > self.size {
            panic!(
                "x ({}) or y ({}) greater than size ({})",
                x, y, self.size
            );
        }

        &mut self.cells[
            (y as usize) * (self.size as usize)
                + (x as usize)
        ]
    }
}
