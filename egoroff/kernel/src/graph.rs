use std::collections::HashMap;

use petgraph::prelude::*;

const SEP: &str = "/";

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct SiteSection {
    pub id: String,
    pub icon: String,
    pub title: String,
    pub descr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<SiteSection>>,
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
        let root = match self.map.get(&root_id) {
            Some(x) => x,
            None => return,
        };

        let root_clone = root.clone();

        let children = match root_clone.children {
            Some(x) => x,
            None => return,
        };

        if children.is_empty() {
            return;
        }

        for child in children.into_iter() {
            let child_id = self.new_node(child);
            self.new_edges(child_id);
            self.g.add_edge(root_id, child_id, 0);
        }
    }

    pub fn get_section(&self, id: &str) -> Option<SiteSection> {
        let node_id = match self.search.get(id) {
            Some(x) => *x,
            None => return None,
        };

        self.map.get(&node_id).cloned()
    }

    pub fn full_path(&self, id: &str) -> String {
        let node_id = match self.search.get(id) {
            Some(x) => *x,
            None => return String::new(),
        };

        let ways = petgraph::algo::all_simple_paths::<Vec<_>, _>(&self.g, 1, node_id, 0, None)
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

    pub fn breadcrumbs(&self, uri: &str) -> (Vec<SiteSection>, String) {
        let root = self.get_section("/").unwrap();
        let mut result = vec![root];
        let mut current = String::from(uri);
        let _: Vec<()> = uri
            .split('/')
            .enumerate()
            .filter(|(_, part)| !part.is_empty())
            .map(|(i, part)| {
                let section = self.get_section(part);
                if let Some(s) = section {
                    if self.full_path(&s.id) != uri {
                        result.push(s);
                    }
                }
                if i == 1 {
                    current = String::from(part);
                }
            })
            .collect();
        (result, current)
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
    
    #[rstest]
    #[case("/")]
    #[case("a")]
    fn get_section_correct(#[case] id: &str) {
        // arrange
        let tg = create_test_data();
        let g = SiteGraph::new(tg);

        // act
        let actual = g.get_section(id);

        // assert
        assert!(actual.is_some());
    }
    
    #[rstest]
    #[case("")]
    #[case("ab")]
    fn get_section_incorrect(#[case] id: &str) {
        // arrange
        let tg = create_test_data();
        let g = SiteGraph::new(tg);

        // act
        let actual = g.get_section(id);

        // assert
        assert!(actual.is_none());
    }

    #[rstest]
    #[case("/a/aa/", "a", 2)]
    #[case("/a/", "a", 1)]
    #[case("/b/", "b", 1)]
    #[case("/b/bb/", "b", 2)]
    #[case("", "", 1)]
    #[case("/", "/", 1)]
    fn breadcrumbs_test(
        #[case] path: &str,
        #[case] expected_current: &str,
        #[case] expected_nodes_count: usize,
    ) {
        // arrange
        let graph = SiteGraph::new(create_test_data());

        // act
        let (nodes, current) = graph.breadcrumbs(path);

        // assert
        assert_eq!(current, expected_current);
        assert_eq!(nodes.len(), expected_nodes_count);
    }

    fn create_test_data() -> SiteSection {
        let aa = SiteSection {
            id: String::from("aa"),
            ..Default::default()
        };

        let bb = SiteSection {
            id: String::from("bb"),
            ..Default::default()
        };

        let a = SiteSection {
            id: String::from("a"),
            children: Some(vec![aa]),
            ..Default::default()
        };

        let b = SiteSection {
            id: String::from("b"),
            children: Some(vec![bb]),
            ..Default::default()
        };

        SiteSection {
            id: String::from("/"),
            children: Some(vec![a, b]),
            ..Default::default()
        }
    }
}
