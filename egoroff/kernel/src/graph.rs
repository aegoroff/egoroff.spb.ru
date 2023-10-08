use std::{collections::HashMap, iter::once};

use itertools::Itertools;
use petgraph::prelude::*;

pub const SEP: &str = "/";

pub const BRAND: &str = "egoroff.spb.ru";

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct SiteSection {
    pub id: String,
    pub icon: String,
    pub title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub descr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<SiteSection>>,
}

impl SiteSection {
    #[must_use]
    pub fn clone_children(&self, active: &str) -> Option<Vec<SiteSection>> {
        self.children.as_ref().map(|sections| {
            sections
                .iter()
                .cloned()
                .map(|mut s| {
                    s.active = Some(s.id == active);
                    s
                })
                .collect()
        })
    }
}

#[derive(Debug)]
pub struct SiteGraph<'a> {
    g: DiGraphMap<i32, ()>,
    next_id: i32,
    map: HashMap<i32, &'a SiteSection>,
    search: HashMap<String, i32>,
}

impl<'a> SiteGraph<'a> {
    #[must_use]
    pub fn new(root: &'a SiteSection) -> Self {
        let mut g = SiteGraph {
            g: DiGraphMap::new(),
            next_id: 1,
            map: HashMap::new(),
            search: HashMap::new(),
        };
        let root_id = g.new_node(root, None);
        g.new_edges(root_id);
        g
    }

    fn new_node(&mut self, s: &'a SiteSection, root_id: Option<i32>) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        self.search.insert(s.id.clone(), id);
        self.map.insert(id, s);
        self.g.add_node(id);
        if let Some(root_id) = root_id {
            self.g.add_edge(root_id, id, ());
        }
        id
    }

    fn new_edges(&mut self, root_id: i32) {
        let Some(root) = self.map.get(&root_id) else {
            return;
        };

        let Some(ref children) = root.children else {
            return;
        };

        if children.is_empty() {
            return;
        }

        for child in children {
            let child_id = self.new_node(child, Some(root_id));
            self.new_edges(child_id);
        }
    }

    #[must_use]
    pub fn get_section(&self, id: &str) -> Option<&SiteSection> {
        let ix = self.search.get(id)?;
        self.map.get(ix).copied()
    }

    #[must_use]
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
                .map(|x| x.id.as_str())
                .filter(|x| *x != SEP)
                .join(SEP);
            format!("{SEP}{path}{SEP}")
        }
    }

    #[must_use]
    pub fn breadcrumbs<'b>(&'b self, uri: &'b str) -> Option<(Vec<&'b SiteSection>, &'b String)> {
        let path = self.to_path(uri)?;

        let mut parent_sections = path.collect_vec();

        let current_ix = usize::from(parent_sections.len() != 1);
        let current = &parent_sections[current_ix].id;
        if parent_sections.len() > 1 && uri.ends_with(SEP) {
            // dont add section root itself to breadcrumbs
            let last = parent_sections[parent_sections.len() - 1];
            let end = format!("{}{SEP}", last.id);
            if uri.ends_with(&end) {
                parent_sections.remove(parent_sections.len() - 1);
            }
        }
        Some((parent_sections, current))
    }

    #[must_use]
    pub fn make_title_path(&self, uri: &str) -> String {
        if uri == SEP || uri.is_empty() {
            return String::new();
        }
        let Some(path) = self.to_path(uri) else {
            return String::new();
        };

        // if section root - skip it in title
        let skip_count = usize::from(uri.ends_with(SEP));

        path.skip(1)
            .collect_vec()
            .into_iter()
            .rev()
            .skip(skip_count)
            .map(|s| s.title.as_str())
            .chain(once(BRAND))
            .join(" | ")
    }

    fn to_path<'b>(&'b self, uri: &'b str) -> Option<impl Iterator<Item = &'b SiteSection>> {
        let root = self.get_section(SEP)?;

        let path = uri
            .split(SEP)
            .filter(|part| !part.is_empty())
            .filter_map(move |part| self.get_section(part));

        Some(once(root).chain(path))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("aa", "/a/aa/")]
    #[case("a", "/a/")]
    #[case("b", "/b/")]
    #[case("bb", "/b/bb/")]
    #[case("ab", "")]
    #[case("/", "/")]
    fn full_path_tests(#[case] id: &str, #[case] expected: &str) {
        // arrange
        let tg = create_test_data();
        let g = SiteGraph::new(&tg);

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
        let g = SiteGraph::new(&tg);

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
        let g = SiteGraph::new(&tg);

        // act
        let actual = g.get_section(id);

        // assert
        assert!(actual.is_none());
    }

    #[rstest]
    #[case("/a/aa/", "a", 2)]
    #[case("/a/aa/1.html", "a", 3)]
    #[case("/a/", "a", 1)]
    #[case("/b/", "b", 1)]
    #[case("/b/bb/", "b", 2)]
    #[case("", "/", 1)]
    #[case("/", "/", 1)]
    fn breadcrumbs_test(
        #[case] path: &str,
        #[case] expected_current: &str,
        #[case] expected_nodes_count: usize,
    ) {
        // arrange
        let root = create_test_data();
        let graph = SiteGraph::new(&root);

        // act
        let (nodes, current) = graph.breadcrumbs(path).unwrap();

        // assert
        assert_eq!(current, expected_current);
        assert_eq!(nodes.len(), expected_nodes_count);
    }

    #[rstest]
    #[case("/a/", "egoroff.spb.ru")]
    #[case("/a/1.html", "a | egoroff.spb.ru")]
    #[case("/b/bb/", "b | egoroff.spb.ru")]
    #[case("/b/bb/1.html", "bb | b | egoroff.spb.ru")]
    #[case("", "")]
    #[case("/", "")]
    fn make_title_path_test(#[case] path: &str, #[case] expected: &str) {
        // arrange
        let root = create_test_data();
        let graph = SiteGraph::new(&root);

        // act
        let actual = graph.make_title_path(path);

        // assert
        assert_eq!(actual, expected);
    }

    fn create_test_data() -> SiteSection {
        let aa = SiteSection {
            id: String::from("aa"),
            title: String::from("aa"),
            ..Default::default()
        };

        let bb = SiteSection {
            id: String::from("bb"),
            title: String::from("bb"),
            ..Default::default()
        };

        let a = SiteSection {
            id: String::from("a"),
            title: String::from("a"),
            children: Some(vec![aa]),
            ..Default::default()
        };

        let b = SiteSection {
            id: String::from("b"),
            title: String::from("b"),
            children: Some(vec![bb]),
            ..Default::default()
        };

        SiteSection {
            id: String::from("/"),
            title: String::from("main"),
            children: Some(vec![a, b]),
            ..Default::default()
        }
    }
}
