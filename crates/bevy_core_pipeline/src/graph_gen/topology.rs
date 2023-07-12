use super::linear::DynAbstractNode;

pub trait UnparsedTopology {}

/// The topology of the graph generated by a [`GraphGenerator`](crate::graph_gen::generator).
/// The node iterator produced by the Topology is the order in which nodes
pub trait Topology {
    fn nodes<'a>(&'a self) -> &'a [&'a DynAbstractNode];

    fn node_labels<'a>(&'a self) -> &'a [&'static str];

    fn edges<'a>(&'a self) -> &'a [()];

    fn source_edges<'a>(&'a self) {}

    fn edges<'a>(
        &'a self,
        existing_sources: &[&'static str],
        existing_target: &[&'static str],
    ) -> &'a [(&'static str, &'static str)] {
        if let Some(edges) = self.edges() {}
    }
}

pub struct Linear {
    nodes: Vec<DynAbstractNode>,
    labels: Vec<&'static str>,
}

impl Linear {
    fn new(nodes: Vec<DynAbstractNode>) {
        let labels = nodes.iter().map(|n| n.label()).collect();
        Self { nodes, labels }
    }
}

impl Topology for Linear {
    fn nodes(&self) -> &[&DynAbstractNode] {
        &self.nodes
    }

    fn node_labels(&self) -> &[&'static str] {
        &self.labels
    }

    fn edges(&self, existing_source: &'static str, existing_target: &'static str) -> usize {
        existing_source
            .into_iter()
            .chain(self.label_sequence.into_iter())
            .chain(existing_target.into_iter())
    }
}
