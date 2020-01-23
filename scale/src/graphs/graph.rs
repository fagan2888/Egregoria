use serde::{Deserialize, Serialize};
use std::collections::hash_map::Keys;
use std::collections::{hash_map, HashMap};
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Edge {
    pub to: NodeID,
    pub weight: f32,
}

type EdgeList = Vec<Edge>;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct NodeID(usize);

#[derive(Serialize, Deserialize)]
pub struct Graph<T> {
    nodes: HashMap<NodeID, T>,
    edges: HashMap<NodeID, EdgeList>,
    backward_edges: HashMap<NodeID, EdgeList>,
    uuid: usize,
}

#[allow(dead_code)]
impl<T> Graph<T> {
    pub fn empty() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            backward_edges: HashMap::new(),
            uuid: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn ids(&self) -> Keys<NodeID, T> {
        self.nodes.keys()
    }

    pub fn get(&self, id: NodeID) -> Option<&T> {
        self.nodes.get(&id)
    }

    pub fn get_mut(&mut self, id: NodeID) -> Option<&mut T> {
        self.nodes.get_mut(&id)
    }

    pub fn push(&mut self, value: T) -> NodeID {
        let uuid = NodeID(self.uuid);
        self.nodes.insert(uuid, value);
        self.edges.insert(uuid, vec![]);
        self.backward_edges.insert(uuid, vec![]);
        self.uuid += 1;
        uuid
    }

    pub fn neighs(&self) -> Vec<(&NodeID, &Edge)> {
        self.edges
            .iter()
            .map(|(from, el)| el.iter().map(move |x| (from, x)))
            .flatten()
            .collect()
    }

    pub fn get_neighs(&self, id: NodeID) -> &EdgeList {
        self.edges.get(&id).expect("Invalid node id")
    }

    pub fn get_backward_neighs(&self, id: NodeID) -> &EdgeList {
        self.backward_edges.get(&id).expect("Invalid node id")
    }

    pub fn set_neighs(&mut self, id: NodeID, neighs: EdgeList) {
        self.edges.insert(id, neighs);
    }

    pub fn add_neigh(&mut self, from: NodeID, to: NodeID, weight: f32) {
        self.edges
            .get_mut(&from)
            .expect("Invalid node id")
            .push(Edge { to, weight });
        self.backward_edges
            .get_mut(&to)
            .expect("Invalid node id")
            .push(Edge { to: from, weight });
    }

    pub fn is_neigh(&self, from: NodeID, to: NodeID) -> bool {
        self.edges[&from].iter().any(|x| x.to == to)
    }

    pub fn remove_neigh(&mut self, from: NodeID, to: NodeID) {
        remove_from_list(&mut self.edges, from, to);
        remove_from_list(&mut self.backward_edges, to, from);
    }

    pub fn remove_outbounds(&mut self, id: NodeID) {
        for Edge { to, .. } in &self.edges[&id] {
            remove_from_list(&mut self.backward_edges, *to, id);
        }
        self.edges.get_mut(&id).unwrap().clear();
    }

    pub fn remove_inbounds(&mut self, id: NodeID) {
        for Edge { to, .. } in &self.backward_edges[&id] {
            remove_from_list(&mut self.edges, *to, id);
        }
        self.backward_edges.get_mut(&id).unwrap().clear();
    }

    pub fn remove_node(&mut self, id: NodeID) {
        self.nodes.remove(&id);
        for x in self.backward_edges.remove(&id).expect("Invalid node id") {
            remove_from_list(&mut self.edges, x.to, id);
        }
        for x in self.edges.remove(&id).expect("Invalid node id") {
            remove_from_list(&mut self.backward_edges, x.to, id);
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.backward_edges.clear();
        self.edges.clear();
        self.uuid = 0;
    }
}

impl<T> Index<NodeID> for Graph<T> {
    type Output = T;

    fn index(&self, index: NodeID) -> &Self::Output {
        &self.nodes[&index]
    }
}

impl<T> IndexMut<NodeID> for Graph<T> {
    fn index_mut(&mut self, index: NodeID) -> &mut Self::Output {
        self.nodes.get_mut(&index).unwrap()
    }
}

impl<T> Index<&NodeID> for Graph<T> {
    type Output = T;

    fn index(&self, index: &NodeID) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T> IndexMut<&NodeID> for Graph<T> {
    fn index_mut(&mut self, index: &NodeID) -> &mut Self::Output {
        self.nodes.get_mut(index).unwrap()
    }
}

impl<'a, T: 'a> IntoIterator for &'a mut Graph<T> {
    type Item = (&'a NodeID, &'a mut T);
    type IntoIter = hash_map::IterMut<'a, NodeID, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.nodes).iter_mut()
    }
}

impl<'a, T: 'a> IntoIterator for &'a Graph<T> {
    type Item = (&'a NodeID, &'a T);
    type IntoIter = hash_map::Iter<'a, NodeID, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.nodes).iter()
    }
}

fn remove_from_list(hash: &mut HashMap<NodeID, EdgeList>, id: NodeID, elem: NodeID) {
    hash.get_mut(&id)
        .expect("Invalid node id")
        .retain(|e| e.to != elem);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph() {
        let mut g = Graph::empty();
        let a = g.push(0);
        let b = g.push(1);
        let c = g.push(2);

        g.add_neigh(a, b, 1.0);

        assert_eq!(g.get_neighs(a).len(), 1);
        assert_eq!(g.get_backward_neighs(b).get(0).unwrap().to, a);

        g.add_neigh(b, c, 1.0);

        g.remove_node(b);

        assert_eq!(g.len(), 2);
        assert_eq!(g.edges.len(), 2);
        assert_eq!(g.backward_edges.len(), 2);
        assert_eq!(g.get_neighs(a).len(), 0);
        assert_eq!(g.get_backward_neighs(c).len(), 0);

        g.add_neigh(a, c, 1.0);
        g.remove_neigh(a, c);

        assert_eq!(g.get_neighs(a).len(), 0);
        assert_eq!(g.get_backward_neighs(c).len(), 0);
    }
}
