use crate::GraphEdgeList;

pub trait Adaptor
where
    Self: Sync + Send,
{
    fn adapt(&self, graph: GraphEdgeList) -> GraphEdgeList;
}

pub struct NoOpAdaptor;

impl Adaptor for NoOpAdaptor {
    fn adapt(&self, graph: GraphEdgeList) -> GraphEdgeList {
        graph
    }
}
