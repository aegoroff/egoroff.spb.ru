#![allow(clippy::module_name_repetitions)]

use itertools::Itertools;
use petgraph::prelude::*;
use std::{collections::HashMap, iter::once, ops::AddAssign};

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
                .map(|s| SiteSection {
                    active: Some(s.id.as_str() == active),
                    keywords: None,
                    children: None,
                    ..s.clone()
                })
                .collect()
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct NodeId(i32);

impl AddAssign<i32> for NodeId {
    fn add_assign(&mut self, rhs: i32) {
        self.0 += rhs;
    }
}

const ROOT_NODE: NodeId = NodeId(1);

#[derive(Debug)]
pub struct SiteGraph<'a> {
    g: DiGraphMap<NodeId, ()>,
    next_id: NodeId,
    map: HashMap<NodeId, &'a SiteSection>,
    search: HashMap<String, NodeId>,
}

impl<'a> SiteGraph<'a> {
    #[must_use]
    pub fn new(root: &'a SiteSection) -> Self {
        let mut g = SiteGraph {
            g: DiGraphMap::new(),
            next_id: ROOT_NODE,
            map: HashMap::new(),
            search: HashMap::new(),
        };
        g.build(root, None);
        g
    }

    fn register(&mut self, s: &'a SiteSection, parent: Option<NodeId>) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.search.insert(s.id.clone(), id);
        self.map.insert(id, s);
        self.g.add_node(id);
        if let Some(parent_id) = parent {
            self.g.add_edge(parent_id, id, ());
        }
        id
    }

    fn build(&mut self, section: &'a SiteSection, parent: Option<NodeId>) {
        let id = self.register(section, parent);
        for child in section.children.iter().flatten() {
            self.build(child, Some(id));
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
            Some(&x) => x,
            None => return String::new(),
        };
        if id == SEP {
            return String::from(SEP);
        }

        let mut path = vec![node_id];
        let mut current = node_id;
        while let Some(parent) = self
            .g
            .neighbors_directed(current, Direction::Incoming)
            .next()
        {
            path.push(parent);
            current = parent;
        }

        let result = path
            .iter()
            .rev()
            .filter_map(|n| self.map.get(n))
            .map(|s| s.id.as_str())
            .filter(|x| *x != SEP)
            .join(SEP);

        format!("{SEP}{result}{SEP}")
    }

    #[must_use]
    pub fn breadcrumbs<'b>(&'b self, uri: &'b str) -> Option<(Vec<&'b SiteSection>, &'b String)> {
        let path = self.to_path(uri)?;

        let mut parent_sections = path.collect_vec();

        let current = &parent_sections.get(1).unwrap_or(&parent_sections[0]).id;
        if parent_sections.len() > 1 && uri.ends_with(SEP) {
            // dont add section root itself to breadcrumbs
            let last = parent_sections[parent_sections.len() - 1];
            // HACK: to decrease allocations. Equal to let end = format!("{}{SEP}", last.id); uri.ends_with(&end)
            if uri[0..uri.len() - 1].ends_with(last.id.as_str()) {
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
    use rstest::{fixture, rstest};

    #[rstest]
    #[case("aa", "/a/aa/")]
    #[case("a", "/a/")]
    #[case("b", "/b/")]
    #[case("bb", "/b/bb/")]
    #[case("ab", "")]
    #[case("/", "/")]
    fn full_path_tests(root: SiteSection, #[case] id: &str, #[case] expected: &str) {
        // arrange
        let g = SiteGraph::new(&root);

        // act
        let actual = g.full_path(id);

        // assert
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case("/")]
    #[case("a")]
    fn get_section_correct(root: SiteSection, #[case] id: &str) {
        // arrange
        let g = SiteGraph::new(&root);

        // act
        let actual = g.get_section(id);

        // assert
        assert!(actual.is_some());
    }

    #[rstest]
    #[case("")]
    #[case("ab")]
    fn get_section_incorrect(root: SiteSection, #[case] id: &str) {
        // arrange
        let g = SiteGraph::new(&root);

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
        root: SiteSection,
        #[case] path: &str,
        #[case] expected_current: &str,
        #[case] expected_nodes_count: usize,
    ) {
        // arrange
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
    fn make_title_path_test(root: SiteSection, #[case] path: &str, #[case] expected: &str) {
        // arrange
        let graph = SiteGraph::new(&root);

        // act
        let actual = graph.make_title_path(path);

        // assert
        assert_eq!(actual, expected);
    }

    #[fixture]
    fn root() -> SiteSection {
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
