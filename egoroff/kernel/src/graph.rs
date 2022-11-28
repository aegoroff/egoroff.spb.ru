use std::hash::Hash;

use petgraph::prelude::*;

#[derive(Debug, Eq, PartialOrd, Ord, Default)]
pub struct SiteSection {
    pub id: String,
    pub icon: String,
    pub title: String,
    pub descr: String,
    pub keywords: String,
    pub active: bool,
    pub children: Vec<SiteSection>,
}

impl PartialEq for SiteSection {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for SiteSection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct Graph<'input> {
    g: DiGraphMap<&'input SiteSection, i32>,
}

impl<'input> Graph<'input> {
    pub fn new(root: &'input mut SiteSection) -> Self {
        let mut g = Graph {
            g: DiGraphMap::new(),
        };
        g.new_node(root);
        g.new_edges(root);
        g
    }

    fn new_node(&mut self, s: &'input SiteSection) {
        self.g.add_node(s);
    }

    fn new_edges(&mut self, root: &'input SiteSection) {
        if root.children.is_empty() {
            return;
        }

        for child in root.children.iter() {
            self.new_node(child);
            self.new_edges(child);
            self.g.add_edge(root, &child, 0);
        }
    }

    pub fn full_path(&self, id: &str) -> String {
        let n = SiteSection { id: String::from(id), ..Default::default() };
        //petgraph::algo::dijkstra(self.g, start, goal, edge_cost);
        String::new()
    }
}
