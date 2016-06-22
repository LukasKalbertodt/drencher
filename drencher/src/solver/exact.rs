//! Exact Solver
//!
//! This solver always finds an optimal solution (with as few moves as
//! possible). **NOTE**: this solver only works for boards of size 16 or less.
//! This is due to some micro-optimization, which for example assumes that
//! the whole board (every cell) can be index with one byte. This means that
//! only 256 (16^2) cells are supported.
//!
//! For more information about the algorithm of this solver, see the comments
//! in the source code.
//!
use board::Board;
use color::Color;
use super::{Solver, Solution};
use std::collections::BTreeMap;
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

const EXPECTED_BRANCHING_FACTOR: usize = 5;

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
        // This is actually necessary...
        if b.is_drenched() {
            return Ok(vec![]);
        }

        // Generate the graph from the board
        let g = generate_graph(&b);
        debug!("initial graph has {} nodes", g.len());

        // Generate sets where each set contains all nodes of a specific color.
        // It's calculated in an inner scope to rebind it immutably.
        let colored_nodes = {
            let mut out = [Set::empty(); 6];

            // Insert each node into the corresponding set
            for node_id in 0..g.len() {
                out[g[node_id].color.tag as usize].insert(node_id);
            }

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
        // outside the loop to reduce the number of allocations.
        let mut new_states = Vec::new();

        // The main loop is a breadth first search through the game tree. Each
        // iteration handles one level. Once we find a valid solution, we know
        // that there is no better solution and just return the found one.
        for depth in 0.. {
            debug!("In depth {} with {} states", depth, states.len());

            // Since we are reusing the old vector, we have to clear it.
            new_states.clear();
            // Preallocate memory for the expected number of new states.
            new_states.reserve(EXPECTED_BRANCHING_FACTOR * states.len());

            // ### ------------------------------------------------------------
            // ### Here we will remove all states that are not needed, but they
            // ### are own strictly less nodes of the graph.
            //
            // We sort the vector first by length of the sets such that the set
            // with the most elements is in the beginning. This gives us a huge
            // speedup, because the following n² algorithm does a lot of
            // subset testing and sorting helps us twofold:
            //
            // For a given set x all possible supersets contain more or as many
            // elements as x. Thus we only have to test with all sets earlier
            // in the vector (plus the ones of the same size).
            //
            // If we test with the bigger sets as possible superset first, we
            // will find a actual superset earlier than the other way around.
            // We expect to remove at least 70% of all states, due to them being
            // subsets of other states. Therefore by sorting we provide a fast
            // path for the common case.
            states.sort_by_key(|state| g.len() - state.owned.len());

            // The actual algorithm to remove is a bit more complicated to
            // understand, but this implementation works without any allocations
            // and does the minimal amount of work. It's still a n² algorithm,
            // but this can't be avoided in the worst case (probably). Most
            // time of the solver is spend in this loop.
            //
            // The algorithm basically partitions the vector. We have three
            // different ranges within the vector:
            //
            // - [0..j] will be kept (aren't a subsets of any other set)
            // - [j..i] will be removed (are subsets of other sets in [0..j])
            // - [i..] aren't checked yet
            //
            let mut j = 0;
            for i in 0..states.len() {
                // Test if we want to keep the current set (states[i]).
                //
                // We have to test if the current set is a subset of any other
                // set. Luckily all possible supersets are in the range [0..j].
                // First, in [i + 1..] are only sets that are smaller or
                // equal in size, thus aren't possible supersets. In [j..i]
                // are only sets that are subsets of sets in [0..j]. Therefore
                // we don't have to consider them possible supersets either,
                // because whenever we find a superset in [j..i] we will also
                // find one in [0..j].
                //
                // Although each set in our vector should be unique, this
                // algorithm would correctly handle duplicates: the first one
                // of the duplicates is kept (because we only consider prior
                // sets as supersets) and the second one is removed.
                if (0..j).all(|a| !states[i].owned.is_subset_of(&states[a].owned)) {
                    // At this point we want to keep states[i], so we swap it
                    // with the element right at the end of the "keep-range".
                    states.swap(i, j);
                    j += 1;
                }
            }

            // Finally we just remove all elements that we don't want to keep
            states.truncate(j);


            // For each node in the game tree, we create the children for the
            // next level.
            for state in &states {
                // First we find out what colors we are adjacent to (we will
                // create a children for each color we are adjacent to).
                let mut adj_colors = ColorSet::new();
                for color in 0..6 {
                    // Here we will check if we can completely remove a color
                    // from the board. This would be perfect move (as in: there
                    // can't be a better move) so we will just try this one
                    // move.
                    // First we have to count the number of nodes with the
                    // given color that we are adjacent to.
                    let num_adj = Set::count_common_elements(
                        &state.adjacent,
                        &colored_nodes[color],
                    );

                    // This will count the number of nodes of the given color
                    // that this state still doesn't own.
                    let num_remaining = Set::count_elements_only_in_first(
                        &colored_nodes[color],
                        &state.owned,
                    );

                    // Now if the number of colored nodes we are adjacent to is
                    // equal to the number of missing nodes of the same color,
                    // we can completely remove that color.
                    let color = Color::new(color as u8);
                    if num_adj == num_remaining && num_adj > 0 {
                        adj_colors.clear();
                        adj_colors.set(color);
                        break;
                    } else if num_adj > 0 {
                        adj_colors.set(color);
                    }
                }

                // For each color we are adjacent to, we have to create a new
                // child in the game tree. Note: also read comments above.
                for color in &adj_colors {
                    // In `colored_adj` we store all adjacent nodes that have
                    // the color `color`.
                    let colored_adj = Set::intersection(
                        &state.adjacent,
                        &colored_nodes[color.tag as usize]
                    );

                    // These are the nodes the we will own after this move.
                    let new_owned = Set::union(&state.owned, &colored_adj);

                    // We have to calculate the new adjacent nodes. These are
                    // the old adjacent nodes plus all nodes that are adjacent
                    // to the colored_adj nodes (that we will soon own) minus
                    // all nodes that we will own.
                    let mut new_adj = state.adjacent.clone();
                    for neighbor_id in &colored_adj {
                        new_adj.union_with(&g[neighbor_id as GraphIndex].adjacent);
                    }
                    new_adj.without(&new_owned);

                    // The new moves are a copy of the old ones plus the
                    // current color.
                    // TODO: avoid second allocation somehow...
                    let mut new_moves = state.moves.clone();
                    new_moves.push(color);

                    // If we are not adjacent to anything onemore, the board
                    // has been drenched and we are done.
                    if new_adj.is_empty() {
                        return Ok(new_moves.to_vec());
                    }

                    // Push the new state onto the vector for the next level.
                    new_states.push(State {
                        moves: new_moves,
                        adjacent: new_adj,
                        owned: new_owned,
                    })
                }
            }

            // Swap the two vectors (in order to reuse the memory)
            mem::swap(&mut states, &mut new_states);
        }
        unreachable!();
    }
}

/// Generates the initial undirected graph representing the board. Every island
/// of multiple cells of the same color are represented by one node. Each node
/// contains the id's of all it's neighbors.
fn generate_graph(b: &Board) -> Graph {
    // We map every cell to the corresponding node to check if we already
    // processed that cell
    let mut map = BTreeMap::new();

    // Create an empty graph. We already allocating enough memory for the worst
    // case.
    let mut g = Graph::default();
    g.nodes.reserve(b.size().pow(2).into());

    // It doesn't matter in which order we progress the cells
    // TODO: maybe it does matter a little bit due to cache misses?
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

/// Our graph is simply a vector of all nodes.
#[derive(Default, Clone)]
struct Graph {
    pub nodes: Vec<Node>,
}

// Custom Debug implementation for debugging purposes
impl fmt::Debug for Graph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Graph ({} nodes) ", self.nodes.len()));
        f.debug_map().entries(self.nodes.iter().enumerate()).finish()
    }
}

impl Graph {
    /// Returns the length of the inner vector as `GraphIndex` to avoid
    /// casting in user code.
    pub fn len(&self) -> GraphIndex {
        self.nodes.len() as GraphIndex
    }
}

// Index operator impls to avoid casting in user code.
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

/// One node in the graph representing the board. It has a color and saves
/// the index of all adjacent nodes.
#[derive(Clone)]
struct Node {
    pub adjacent: Set,
    pub color: Color,
}

// Custom debug impl for debugging purposes
impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}> --> {:?}", self.color, self.adjacent)
    }
}

/// This type represents a set and mirrors the functionality of `BitSet`. The
/// difference is that the set data of `InlineBitSet` is stored inline (on
/// the stack). This is supposed to decrease cache misses and memory usage.
/// Usage of this instead of `BitSet` lead to a approximately 10x speedup.
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
        // TODO: maybe it's faster to cache the length (probably not). Measure!
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
}
