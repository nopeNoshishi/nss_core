// Std
use std::collections::{HashSet, VecDeque};

// External
use anyhow::Result;

// Internal
use crate::repository::NssRepository;

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct VertexIndex(usize);

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct EdgeIndex(usize);

#[derive(Debug, PartialEq)]
struct Vertex<T: PartialEq> {
    value: T,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Edge {
    start_id: VertexIndex,
    end_id: VertexIndex,
}

#[derive(Debug)]
pub struct Graph<T: PartialEq> {
    vertexs: Vec<Vertex<T>>,
    edges: Vec<Edge>,
}

impl<T: Clone + PartialEq + Eq + std::hash::Hash> Graph<T> {
    pub fn new() -> Self {
        Graph {
            vertexs: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, value: T) -> VertexIndex {
        let new_vertex = Vertex { value };

        // indexing for new vertex
        let new_id = self
            .vertexs
            .iter()
            .position(|x| x == &new_vertex)
            .map(VertexIndex)
            .unwrap_or_else(|| {
                let index = VertexIndex(self.vertex_count());
                self.vertexs.push(new_vertex);
                index
            });

        new_id
    }

    pub fn add_edges(&mut self, start_id: VertexIndex, end_id: VertexIndex) -> EdgeIndex {
        let new_edge = Edge { start_id, end_id };

        // indexing for new vertex
        let new_id = EdgeIndex(self.efge_count());
        self.edges.push(new_edge);

        new_id
    }

    fn get_vertex_id(&self, value: &T) -> Option<VertexIndex> {
        let vertex = Vertex {
            value: value.clone(),
        };

        // indexing for new vertex
        let new_id = self
            .vertexs
            .iter()
            .position(|x| x == &vertex)
            .map(VertexIndex);

        new_id
    }

    pub fn get_vertex_value(&self, id: VertexIndex) -> Option<&T> {
        self.vertexs.get(id.0).map(|n| &n.value)
    }

    fn vertex_count(&self) -> usize {
        self.vertexs.len()
    }

    fn efge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn distance(&self, start_value: &T, end_value: &T) -> Option<usize> {
        let start_id = self.get_vertex_id(start_value);
        let end_id = self.get_vertex_id(end_value);

        if start_id.is_none() || end_id.is_none() {
            return None;
        }

        let mut visited = vec![false; self.vertexs.len()];
        let mut queue: VecDeque<(VertexIndex, usize)> = VecDeque::new(); // <vertex, distance>
        visited[start_id.unwrap().0] = true;
        queue.push_back((start_id.unwrap(), 0));

        while let Some((current_vertex, distance)) = queue.pop_front() {
            if current_vertex == end_id.unwrap() {
                return Some(distance);
            }

            for edge in &self.edges {
                if edge.start_id == current_vertex && !visited[edge.end_id.0] {
                    visited[edge.end_id.0] = true;
                    queue.push_back((edge.end_id, distance + 1));
                }
            }
        }

        None
    }

    fn to_value_set(&self) -> HashSet<&T> {
        self.vertexs.iter().map(|v| &v.value).collect()
    }

    pub fn common_vertex_value<'a>(&'a self, another_graph: &'a Graph<T>) -> Option<&T> {
        let vertexs_set = self.to_value_set();
        let t_vertexs_set = another_graph.to_value_set();

        let graph_start = &self.vertexs[0].value;
        let another_graph_start = &another_graph.vertexs[0].value;

        vertexs_set
            .intersection(&t_vertexs_set)
            .map(|x| {
                let mut distance = 0;

                distance += self.distance(graph_start, x).unwrap();
                distance += another_graph.distance(another_graph_start, x).unwrap();
                (distance, x)
            })
            .min_by_key(|(num, _)| *num)
            .map(|x| *x.1)
    }
}

impl<T: Clone + PartialEq + Eq + std::hash::Hash> Default for Graph<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub type CommitHash = String;

pub type CommitGraph = Graph<CommitHash>;

impl CommitGraph {
    pub fn build(start_hash: String, repo: &NssRepository, deep: usize) -> Result<Self> {
        let mut graph = Graph::<CommitHash>::new();
        Self::commit_history(&mut graph, start_hash, repo, 0, deep)?;

        Ok(graph)
    }

    fn commit_history(
        graph: &mut Graph<CommitHash>,
        current_hash: String,
        repo: &NssRepository,
        current_depth: usize,
        max_depth: usize,
    ) -> Result<()> {
        if current_depth >= max_depth {
            return Ok(());
        }

        let commit = repo.objects().read_commit(&current_hash)?;

        let child_id = graph.add_vertex(current_hash);

        for parent_hash in commit.parents {
            let parent_id = graph.add_vertex(parent_hash.clone());
            graph.add_edges(child_id, parent_id);

            Self::commit_history(graph, parent_hash, repo, current_depth + 1, max_depth)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_vertex_value() {
        let mut graph = Graph::<CommitHash>::new();

        let v7_id = graph.add_vertex("v7".to_string());
        let v4_id = graph.add_vertex("v4".to_string());
        let v3_id = graph.add_vertex("v3".to_string());
        let v2_id = graph.add_vertex("v2".to_string());
        let v1_id = graph.add_vertex("v1".to_string());

        let _ = graph.add_edges(v7_id, v4_id);
        let _ = graph.add_edges(v4_id, v3_id);
        let _ = graph.add_edges(v4_id, v2_id);
        let _ = graph.add_edges(v2_id, v1_id);

        let mut another_graph = Graph::<CommitHash>::new();
        let v5_id = another_graph.add_vertex("v5".to_string());
        let v2_id = another_graph.add_vertex("v2".to_string());
        let v1_id = another_graph.add_vertex("v1".to_string());
        let _ = another_graph.add_edges(v5_id, v2_id);
        let _ = another_graph.add_edges(v2_id, v1_id);

        let common = graph.common_vertex_value(&another_graph);

        assert_eq!(common, Some(&"v2".to_string()));
    }

    #[test]
    fn test_graph_2() {}
}
