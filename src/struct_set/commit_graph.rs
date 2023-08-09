// External
use anyhow::Result;

// Internal
use crate::repository::NssRepository;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct VertexIndex(usize);

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct EdgeIndex(usize);

#[derive(Debug, PartialEq)]
struct Vertex<T: PartialEq> {
    value: T
}

#[allow(dead_code)]
#[derive(Debug)]
struct Edge {
    start_id: VertexIndex,
    end_id: VertexIndex
}

#[derive(Debug)]
pub struct Graph<T: PartialEq> {
    vertexs: Vec<Vertex<T>>,
    edges: Vec<Edge>
}

impl<T: PartialEq> Graph<T> {
    pub fn new() -> Self {
        Graph {
            vertexs: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, value: T) -> VertexIndex {

        let new_vertex = Vertex {
            value
        };

        // indexing for new vertex
        let new_id = self.vertexs.iter()
            .position(|x| x == &new_vertex)
            .map(|x| VertexIndex {0: x})
            .unwrap_or_else( || {
                let index = VertexIndex { 0: self.vertex_count() };
                self.vertexs.push(new_vertex);
                index
            });

        new_id
    }

    pub fn add_edges(&mut self, start_id: VertexIndex, end_id: VertexIndex) -> EdgeIndex {

        let new_edge = Edge {
            start_id,
            end_id
        };

        // indexing for new vertex
        let new_id = EdgeIndex { 0: self.efge_count() };
        self.edges.push(new_edge);

        new_id
    }

    pub fn get_vertex_value(&self, id: VertexIndex) -> Option<&T> {
        self.vertexs.get(id.0).map(|n| &n.value)
    }

    pub fn vertex_count(&self) -> usize {
        self.vertexs.len()
    }

    pub fn efge_count(&self) -> usize {
        self.edges.len()
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

        let commit = repo.read_commit(&current_hash)?;
        let child_id = graph.add_vertex(current_hash);

        for parent_hash in commit.parents {
            let parent_id = graph.add_vertex(parent_hash.clone());
            graph.add_edges(child_id.clone(), parent_id.clone());

            Self::commit_history(graph, parent_hash, repo, current_depth + 1, max_depth)?;
        }

        Ok(())
    }

    // fn to_hash_vec(&self) -> Vec<&CommitHash> {
    //     self.vertexs.iter().map(|v| &v.value).collect()
    // }

    // pub fn common_ancester(self, another_graph: CommitGraph) -> CommitHash {
    //     "String".to_string()
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::struct_set::Commit;

    #[test]
    fn test_graph() {

        let first_commit = Commit::new(
            "ko093".to_string(),
            vec![],
            "nopeNoshihsi".to_string(),
            "nopeNoshihsi".to_string(),
            " initial".to_string()
            ).unwrap();
        
        let mut graph = Graph::<Commit>::new();

        let id = graph.add_vertex(first_commit);

        let commit = graph.get_vertex_value(id);

        println!("{:?}", commit);
    }
}

