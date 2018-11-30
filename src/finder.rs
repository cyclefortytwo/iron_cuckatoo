use hashbrown::HashMap;
use std::time::SystemTime;

#[derive(Clone)]
pub struct Solution {
    pub nonces: Vec<u32>,
}

pub struct Graph {
    adj_index: HashMap<u32, usize>,
    adj_store: Vec<AdjNode>,
    nonces: HashMap<(u32, u32), u32>,
}

struct Search {
    length: usize,
    path: Vec<u32>,
    solutions: Vec<Solution>,

    state: HashMap<u32, NodeState>,
    node_visited: usize,
    node_explored: usize,
}

#[derive(Clone, Copy)]
enum NodeState {
    NotVisited,
    Visited,
    Explored,
}

impl Search {
    fn new(node_count: usize, length: usize) -> Search {
        Search {
            path: Vec::with_capacity(node_count),
            solutions: vec![],
            length: length * 2,
            state: HashMap::with_capacity_and_hasher(node_count, Default::default()),
            node_visited: 0,
            node_explored: 0,
        }
    }

    #[inline]
    fn visit(&mut self, node: u32) {
        self.state.insert(node, NodeState::Visited);
        self.path.push(node);
        self.node_visited += 1;
    }

    #[inline]
    fn explore(&mut self, node: u32) {
        self.state.insert(node, NodeState::Explored);
        self.path.push(node);
        self.node_explored += 1;
    }

    #[inline]
    fn leave(&mut self) {
        self.path.pop();
    }

    #[inline]
    fn state(&self, node: u32) -> NodeState {
        match self.state.get(&node) {
            None => NodeState::NotVisited,
            Some(state) => *state,
        }
    }

    #[inline]
    fn is_visited(&self, node: u32) -> bool {
        match self.state(node) {
            NodeState::NotVisited => false,
            _ => true,
        }
    }

    #[inline]
    fn is_explored(&self, node: u32) -> bool {
        match self.state(node) {
            NodeState::Explored => true,
            _ => false,
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.path.clear();
        //self.state.clear();
    }

    fn is_cycle(&mut self, node: u32, is_first: bool) -> bool {
        //  TODO remove after tests
        if self.path.contains(&node) {
            let pos = self.path.iter().position(|&v| v == node).unwrap();
            let diff = (self.path.len() - pos) / 2;
            if diff > 1 {
                println!("Found {}-cycle {}", diff, node);
            }
        }
        let res =
            self.path.len() > self.length - 1 && self.path[self.path.len() - self.length] == node;
        if res && !is_first {
            self.path.push(node);
        }
        res
    }
}

struct AdjNode {
    value: u32,
    next: Option<usize>,
}

impl AdjNode {
    #[inline]
    fn first(value: u32) -> AdjNode {
        AdjNode { value, next: None }
    }

    #[inline]
    fn next(value: u32, next: usize) -> AdjNode {
        AdjNode {
            value,
            next: Some(next),
        }
    }
}

struct AdjList<'a> {
    current: Option<&'a AdjNode>,
    adj_store: &'a Vec<AdjNode>,
}

impl<'a> AdjList<'a> {
    #[inline]
    pub fn new(current: Option<&'a AdjNode>, adj_store: &'a Vec<AdjNode>) -> AdjList<'a> {
        AdjList { current, adj_store }
    }
}

impl<'a> Iterator for AdjList<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(node) => {
                let val = node.value;
                match node.next {
                    None => self.current = None,
                    Some(next_index) => self.current = Some(&self.adj_store[next_index]),
                }
                Some(val)
            }
        }
    }
}

fn nonce_key(node1: u32, node2: u32) -> (u32, u32) {
    if node1 < node2 {
        (node1, node2)
    } else {
        (node2, node1)
    }
}

impl Graph {
    pub fn build(edges: &[u32]) -> Graph {
        let edge_count = edges[1] as usize;
        let mut g = Graph {
            adj_index: HashMap::with_capacity_and_hasher(edge_count * 2, Default::default()),
            nonces: HashMap::with_capacity_and_hasher(edge_count, Default::default()),
            adj_store: Vec::with_capacity(edge_count * 2),
        };
        const STEP: usize = 4;
        for i in 1..=edge_count {
            let n1 = edges[i * STEP];
            let n2 = edges[i * STEP + 1];
            let nonce = edges[i * STEP + 2];
            g.add_edge(n1, n2);
            g.nonces.insert(nonce_key(n1, n2), nonce);
        }
        g
    }

    fn get_nonce(&self, node1: u32, node2: u32) -> Result<u32, String> {
        match self.nonces.get(&nonce_key(node1, node2)) {
            None => Err(format!("can not find  a nonce for {}:{}", node1, node2)),
            Some(v) => Ok(*v),
        }
    }

    #[inline]
    pub fn node_count(&self) -> usize {
        self.adj_index.len()
    }

    #[inline]
    pub fn edge_count(&self) -> usize {
        self.adj_store.len() / 2
    }

    #[inline]
    fn add_edge(&mut self, node1: u32, node2: u32) {
        self.add_half_edge(node1, node2);
        self.add_half_edge(node2, node1);
    }

    fn add_half_edge(&mut self, from: u32, to: u32) {
        if let Some(index) = self.adj_index.get(&from) {
            self.adj_store.push(AdjNode::next(to, *index));
        } else {
            self.adj_store.push(AdjNode::first(to));
        }
        self.adj_index.insert(from, self.adj_store.len() - 1);
    }

    fn neighbors(&self, node: u32) -> Option<impl Iterator<Item = u32> + '_> {
        let node = match self.adj_index.get(&node) {
            Some(index) => Some(&self.adj_store[*index]),
            None => return None,
        };
        Some(AdjList::new(node, &self.adj_store))
    }

    #[inline]
    fn nodes(&self) -> impl Iterator<Item = &u32> {
        self.adj_index.keys()
    }

    pub fn find(&self) -> Result<Vec<Solution>, String> {
        let mut search = Search::new(self.node_count(), 42);
        for node in self.nodes() {
            self.walk_graph(*node, &mut search)?;
            search.clear();
        }
        println!("Explored nodes: {}", search.node_explored);
        println!("Found cycles: {}", search.solutions.len());
        Ok(search.solutions.clone())
    }

    fn add_solution(&self, s: &mut Search) -> Result<(), String> {
        let res: Result<Vec<_>, _> = s.path[s.path.len() - s.length..]
            .chunks(2)
            .map(|pair| match pair {
                &[n1, n2] => self.get_nonce(n1, n2),
                _ => Err("not an edge".to_string()),
            })
            .collect();
        let mut nonces = match res {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Failed to get nonce {:?}", e));
            }
        };
        nonces.sort();
        let sol = Solution { nonces };
        s.solutions.push(sol);
        Ok(())
    }

    fn walk_graph(&self, current: u32, search: &mut Search) -> Result<(), String> {
        if search.is_explored(current) {
            if search.is_cycle(current, true) {
                self.add_solution(search)?;
            }
            return Ok(());
        }

        let neighbors = match self.neighbors(current) {
            None => return Ok(()),
            Some(it) => it,
        };
        search.explore(current);
        for ns in neighbors {
            if !search.is_visited(ns) {
                search.visit(ns);
                self.walk_graph(ns ^ 1, search)?;
                search.leave();
            } else {
                if search.is_cycle(ns, false) {
                    self.add_solution(search)?;
                }
            }
        }
        search.leave();
        Ok(())
    }
}
