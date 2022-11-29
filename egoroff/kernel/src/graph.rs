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
pub struct SiteGraph<'input> {
    g: DiGraphMap<&'input SiteSection, i32>,
}

impl<'input> SiteGraph<'input> {
    pub fn new(root: &'input SiteSection) -> Self {
        let mut g = SiteGraph {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("aa", "/a/aa/")]
    #[case("a", "/a/")]
    #[case("b", "/b/")]
    #[case("bb", "/b/bb/")]
    #[case("ab", "")]
    #[case("/", "/")]
    fn full_path(#[case] id: &str, #[case] expected: &str) {
        // arrange
        let tg = create_test_data();
        let g = SiteGraph::new(&tg);

        // act
        let actual = g.full_path(id);

        // assert
        assert_eq!(actual, expected);
    }

    fn create_test_data() -> SiteSection {
        let mut root = SiteSection {
            id: String::from("/"),
            children: Vec::new(),
            ..Default::default()
        };
        
        let mut a = SiteSection {
            id: String::from("a"),
            children: Vec::new(),
            ..Default::default()
        };
        
        let mut b = SiteSection {
            id: String::from("b"),
            children: Vec::new(),
            ..Default::default()
        };
        
        let aa = SiteSection {
            id: String::from("aa"),
            children: Vec::new(),
            ..Default::default()
        };
        
        let bb = SiteSection {
            id: String::from("bb"),
            children: Vec::new(),
            ..Default::default()
        };

        a.children.push(aa);
        b.children.push(bb);
        root.children.push(a);
        root.children.push(b);
        root
    }
}