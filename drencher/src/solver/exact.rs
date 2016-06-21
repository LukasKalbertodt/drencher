//! Exact Solver
//!
//! This solver always finds an optimal solution (with as few moves as
//! possible). **NOTE**: this solver only works for boards of size 16 or less.
//! This is due to some micro-optimization, which for example assumes that
//! the whole board (every cell) can be index with one byte. This means that
//! only 256 (16^2) cells are supported.
//!
//!
//!
// TODO: Complete documentation above
use board::Board;
use color::Color;
use super::{Solver, Solution};
use std::collections::HashMap;
use std::fmt;
use std::ops;
use smallvec::SmallVec;
use std::mem;
use util::{CellMap, ColorSet};

/// Type definition of exact solver. See module documentation for more
/// information.
pub struct Exact;

type GraphIndex = u8;
type Pos = (u8, u8);
type Set = InlineBitSet;

/// Used to represent one node in the game tree. See module documentation for
/// more information.
#[derive(Clone)]
struct State {
    pub moves: SmallVec<[Color; 16]>,
    pub adjacent: Set,
    pub owned: Set,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "moves: ["));
        for m in &*self.moves {
            try!(write!(f, "{}", m));
        }
        write!(f, "] @ adj {:?}", self.adjacent)
    }
}

impl Solver for Exact {
    fn solve(&self, b: Board) -> Result<Solution, Solution> {
        // Generate the graph from the board
        let g = generate_graph(&b);
        debug!("initial graph has {} nodes", g.len());

        // Generate sets which contain all nodes of a specific color
        let nodes_with_color = {
            // fixed size array
            let mut out = [Set::empty(); 6];

            // fill the sets
            for node_id in 0..g.len() {
                out[g[node_id].color.tag as usize].insert(node_id);
            }
            // for color in 0..6 {
            //     let nodes = g.nodes.iter()
            //         .enumerate()
            //         .filter(|&(_, n)| n.color == Color::new(color))
            //         .map(|(idx, _)| idx);
            //     out[color as usize].extend(nodes);
            // }
            out
        };

        // The initial state: no moves yet, the same adjacent nodes as the
        // first node of the graph (top left) and only the first node owned.
        let mut states = vec![State {
            moves: SmallVec::new(),
            adjacent: g[0].adjacent.clone(),
            owned: Set::with_only_first(),
        }];

        // We will collect the new level of the game tree in here. We keep it
        // outside the loop to reduce number of allocations.
        let mut new_states = Vec::new();

        loop {
            // Since we are reusing the old vector, we have to clear it
            new_states.clear();
            // We will need some capacity... TODO: we should test this
            // new_states.reserve(5 * states.len());

            // check the relationship between states
            // let before = ::time::now();
            // println!("---------");
            // print!("From {} -> ", states.len());
            let mut j = 0;

            states.sort_by_key(|state| g.len() - state.owned.len());
            // let after_sort = ::time::now();
            for i in 0..states.len() {
                // print!("i => ");
                // invariants:
                // items v[0..j] will be kept
                // items v[j..i] will be removed
                if (0..j).all(|a| !states[i].owned.is_subset_of(&states[a].owned)) {
                    // println!("swapping! (j = {} now)", j + 1);
                    states.swap(i, j);
                    j += 1;
                }
            }
            states.truncate(j);
            // println!(
            //     "{} (in {}, sort: {})",
            //     states.len(),
            //     ::time::now() - before,
            //     after_sort - before
            // );


            // For each node in the game tree, we create new children in the
            // next level.
            for state in &states {
                // First we find out what colors we are adjacent to
                let mut adj_colors = ColorSet::new();
                for color in 0..6 {
                    let color = Color::new(color);

                    // let num_adj = state.adjacent
                    //     .intersection(&nodes_with_color[color.tag as usize])
                    //     .count();

                    let num_adj = Set::count_common_elements(
                        &state.adjacent,
                        &nodes_with_color[color.tag as usize],
                    );

                    let num_remaining = Set::count_elements_only_in_first(
                        &nodes_with_color[color.tag as usize],
                        &state.owned,
                    );

                    if num_adj == num_remaining && num_adj > 0 {
                        adj_colors.clear();
                        adj_colors.set(color);
                        break;
                    } else if num_adj > 0 {
                        adj_colors.set(color);
                    }
                }

                // For each color we are adjacent to, we have to create a new
                // child in the game tree
                for color in &adj_colors {
                    // In `active_adj` we store all adjacent nodes that have
                    // the color `color`.
                    let active_adj = Set::intersection(
                        &state.adjacent,
                        &nodes_with_color[color.tag as usize]
                    );

                    let mut new_adj = state.adjacent.clone();
                    let new_owned = Set::union(&state.owned, &active_adj);

                    for neighbor_id in &active_adj {
                        new_adj.union_with(&g[neighbor_id as GraphIndex].adjacent);
                    }

                    new_adj.without(&new_owned);

                    let mut new_moves = state.moves.clone();
                    new_moves.push(color);

                    if new_adj.is_empty() {
                        return Ok(new_moves.to_vec());
                    }

                    // // check if the current move is strictly worse than another
                    // // move
                    // let not_needed = new_states.iter().any(|state: &State|
                    //     new_owned.is_subset(&state.owned)
                    // );
                    if true {
                        new_states.push(State {
                            moves: new_moves,
                            adjacent: new_adj,
                            owned: new_owned,
                        })
                    }
                }
            }

            // debug!("{:#?}", new_states);
            // debug!("{:#?}", g);

            // states = new_states;
            mem::swap(&mut states, &mut new_states);
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
            let new_id = g.len();
            g.nodes.push(Node {
                adjacent: Set::empty(),
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
                    g[id].adjacent.insert(new_id);
                    g[new_id].adjacent.insert(id);
                }
            }
        }
    }

    g.nodes.shrink_to_fit();

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
struct Graph {
    pub nodes: Vec<Node>,
}

impl fmt::Debug for Graph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Graph ({} nodes) ", self.nodes.len()));
        f.debug_map().entries(self.nodes.iter().enumerate()).finish()
    }
}

impl Graph {
    pub fn len(&self) -> GraphIndex {
        self.nodes.len() as GraphIndex
    }
}


impl ops::Index<GraphIndex> for Graph {
    type Output = Node;
    fn index(&self, idx: GraphIndex) -> &Self::Output {
        &self.nodes[idx as usize]
    }
}
impl ops::IndexMut<GraphIndex> for Graph {
    fn index_mut(&mut self, idx: GraphIndex) -> &mut Self::Output {
        &mut self.nodes[idx as usize]
    }
}

#[derive(Clone)]
struct Node {
    pub adjacent: Set,
    pub color: Color,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}> --> {:?}", self.color, self.adjacent)
    }
}

/// This type represents a set and mirrors the functionality of `BitSet`. The
/// difference is that the set data of `InlineBitSet` is stored inline (on
/// the stack). This is supposed to decrease cache misses and memory usage.
///
/// This type is specialized for the task at hand: it can't grow and only
/// offers functionality important for the solver. It also imposes one main
/// limitation to the solver: we can only handle integer keys up to 255. This
/// means that we can't represent more than 256 nodes in our graph and thus are
/// limited to boards of the size 16 or less.
#[derive(Clone, Copy, PartialEq, Eq)]
struct InlineBitSet {
    data: [u64; 4],
}

impl InlineBitSet {
    pub fn empty() -> Self {
        InlineBitSet {
            data: [0, 0, 0, 0],
        }
    }

    pub fn with_only_first() -> Self {
        let mut out = Self::empty();
        out.insert(0);
        out
    }

    pub fn len(&self) -> u8 {
        self.data.iter().fold(0, |acc, block| acc + block.count_ones() as u8)
    }

    pub fn is_empty(&self) -> bool {
        self.data.iter().all(|&block| block == 0)
    }

    pub fn contains(&self, query: u8) -> bool {
        // We save 64 values per block (by using u64's). Here we determine
        // what block the query lives in.
        let block = &self.data[query as usize / 64];

        // Check if the corresponding bit is set.
        block & (1 << query % 64) != 0
    }

    pub fn insert(&mut self, elem: u8) {
        // See `contains` for further details
        let block = &mut self.data[elem as usize / 64];

        // We set a single 1 at the corresponding position.
        *block |= 1 << (elem % 64);
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.data.iter()
            .zip(&other.data)
            .all(|(&this, &other)| this & other == this)
    }

    pub fn count_common_elements(a: &Self, b: &Self) -> u8 {
        a.data.iter()
            .zip(&b.data)
            .fold(0, |acc, (&a, &b)| {
                acc + (a & b).count_ones() as u8
            })
    }

    pub fn count_elements_only_in_first(a: &Self, b: &Self) -> u8 {
        a.data.iter()
            .zip(&b.data)
            .fold(0, |acc, (&a, &b)| {
                acc + (a ^ (b & b)).count_ones() as u8
            })
    }

    pub fn union_with(&mut self, other: &Self) {
        for (this, &other) in self.data.iter_mut().zip(&other.data) {
            *this |= other;
        }
    }

    pub fn intersect_with(&mut self, other: &Self) {
        for (this, &other) in self.data.iter_mut().zip(&other.data) {
            *this &= other;
        }
    }

    pub fn without(&mut self, other: &Self) {
        for (this, &other) in self.data.iter_mut().zip(&other.data) {
            *this ^= *this & other;
        }
    }

    pub fn union(a: &Self, b: &Self) -> Self {
        let mut a = *a;
        a.union_with(b);
        a
    }

    pub fn intersection(a: &Self, b: &Self) -> Self {
        let mut a = *a;
        a.intersect_with(b);
        a
    }
}

impl fmt::Debug for InlineBitSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<'a> IntoIterator for &'a InlineBitSet {
    type Item = u8;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            set: *self,
            pos: 0,
        }
    }
}

struct Iter {
    set: InlineBitSet,
    pos: u16,
}

impl Iterator for Iter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos <= u8::max_value().into() && !self.set.contains(self.pos as u8) {
            self.pos += 1;
        }

        match self.pos {
            0 ... 255 => {
                self.pos += 1;
                Some((self.pos - 1) as u8)
            }
            _ => None,
        }
    }

    // TODO: maybe add specializations
}
