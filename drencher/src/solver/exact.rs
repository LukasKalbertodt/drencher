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
// TODO: Edit documentation above
use board::Board;
use color::Color;
use super::{Solver, Solution};
use std::collections::{HashMap, HashSet};
use std::iter::repeat;
use std::fmt;
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
    pub curr_node: usize,
    pub owned: Vec<usize>,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "moves: ["));
        for m in &self.moves {
            try!(write!(f, "{}", m));
        }
        write!(f, "] @ node #{}", self.curr_node)
    }
}

impl Solver for Exact {
    fn solve(&self, b: Board) -> Result<Solution, Solution> {
        let mut g = generate_graph(&b);
        debug!("initial graph has {} nodes", g.nodes.len());

        let mut states = vec![State {
            moves: vec![],
            curr_node: 0,
            owned: vec![0],
        }];

        loop {
            let mut new_states = Vec::new();

            for state in &states {

                for color in 0..6 {
                    let color = Color::new(color);


                    let (new_adj, new_owned) = {
                        let n = &g.nodes[state.curr_node];
                        if n.adjacent.iter().all(|&id| g.nodes[id].color != color) {
                            continue;
                        }

                        let mut new_adj: HashSet<_> = n.adjacent.iter().cloned().collect();
                        let mut new_owned = state.owned.clone();

                        for &neighbor_id in &n.adjacent {
                            let neighbor = &g.nodes[neighbor_id];

                            if neighbor.color == color {
                                new_adj.extend(neighbor.adjacent.iter().cloned());
                                new_adj.remove(&neighbor_id);
                                new_owned.push(neighbor_id);
                            }
                        }

                        for owned in &state.owned {
                            new_adj.remove(owned);
                        }
                        (new_adj, new_owned)
                    };

                    let mut new_moves = state.moves.clone();
                    new_moves.push(color);

                    if new_adj.is_empty() {
                        return Ok(new_moves);
                    }

                    g.nodes.push(Node {
                        color: color,
                        adjacent: new_adj.iter().cloned().collect(),
                    });


                    new_states.push(State {
                        moves: new_moves,
                        curr_node: g.nodes.len() - 1,
                        owned: new_owned,
                    })
                }
            }

            debug!("{:#?}", new_states);
            debug!("{:#?}", g);

            states = new_states;
        }
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

#[derive(Default, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
}

impl fmt::Debug for Graph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Graph ({} nodes) ", self.nodes.len()));
        f.debug_map().entries(self.nodes.iter().enumerate()).finish()
    }
}

#[derive(Clone)]
pub struct Node {
    pub adjacent: Vec<usize>,
    pub color: Color,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}> --> {:?}", self.color, self.adjacent)
    }
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
