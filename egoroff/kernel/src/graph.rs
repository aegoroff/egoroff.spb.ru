use std::collections::HashMap;

use petgraph::prelude::*;

const SEP: &str = "/";

#[derive(Debug, Default, Clone, Deserialize)]
pub struct SiteSection {
    pub id: String,
    pub icon: String,
    pub title: String,
    pub descr: String,
    pub keywords: String,
    pub active: bool,
    pub children: Vec<SiteSection>,
}

#[derive(Debug, Clone)]
pub struct SiteGraph {
    g: DiGraphMap<i32, i32>,
    next_id: i32,
    map: HashMap<i32, SiteSection>,
    search: HashMap<String, i32>,
}

impl SiteGraph {
    pub fn new(root: SiteSection) -> Self {
        let mut g = SiteGraph {
            g: DiGraphMap::new(),
            next_id: 1,
            map: HashMap::new(),
            search: HashMap::new(),
        };
        let root_id = g.new_node(root);
        g.new_edges(root_id);
        g
    }

    fn new_node(&mut self, s: SiteSection) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        self.search.insert(s.id.clone(), id);
        self.map.insert(id, s);
        self.g.add_node(id);
        id
    }

    fn new_edges(&mut self, root_id: i32) {
        let Some(root) = self.map.get(&root_id) else { return };
        let root_clone = root.clone();

        if root_clone.children.is_empty() {
            return;
        }

        for child in root_clone.children.into_iter() {
            let child_id = self.new_node(child);
            self.new_edges(child_id);
            self.g.add_edge(root_id, child_id, 0);
        }
    }

    pub fn full_path(&self, id: &str) -> String {
        let Some(node_id) = self.search.get(id) else { return String::new() };

        let ways = petgraph::algo::all_simple_paths::<Vec<_>, _>(&self.g, 1, *node_id, 0, None)
            .collect::<Vec<_>>();
        if ways.is_empty() {
            if id == SEP {
                String::from(SEP)
            } else {
                String::new()
            }
        } else {
            let way = &ways[0];
            let path = way
                .iter()
                .filter_map(|s| self.map.get(s))
                .map(|x| x.id.clone())
                .fold(String::from(""), |acc, x| {
                    if x == SEP {
                        String::new()
                    } else {
                        format!("{acc}{SEP}{x}")
                    }
                });
            format!("{path}{SEP}")
        }
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
        let g = SiteGraph::new(tg);

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
