use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct NodeParams {
    pub volume: f32,
    pub pan: f32,
    pub tempo: f32,
}

impl Default for NodeParams {
    fn default() -> Self {
        NodeParams { volume: 1.0, pan: 0.0, tempo: 120.0 }
    }
}

pub trait AudioNode: Send {
    fn process(&mut self, inputs: &[&[f32]], output: &mut [f32], params: &NodeParams);
    fn name(&self) -> &str;
}

pub type NodeId = NodeIndex;

#[derive(Debug)]
pub enum GraphError {
    CycleDetected,
    NodeNotFound,
    InvalidConnection,
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::CycleDetected => write!(f, "Cycle detected in audio graph"),
            GraphError::NodeNotFound => write!(f, "Node not found"),
            GraphError::InvalidConnection => write!(f, "Invalid connection"),
        }
    }
}

impl std::error::Error for GraphError {}

pub struct SourceNode {
    phase: f32,
    frequency: f32,
    sample_rate: f32,
}

impl SourceNode {
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        SourceNode { phase: 0.0, frequency, sample_rate }
    }
}

impl AudioNode for SourceNode {
    fn process(&mut self, _inputs: &[&[f32]], output: &mut [f32], _params: &NodeParams) {
        use std::f32::consts::PI;
        let phase_increment = 2.0 * PI * self.frequency / self.sample_rate;
        for sample in output.iter_mut() {
            *sample = self.phase.sin();
            self.phase += phase_increment;
            if self.phase > 2.0 * PI { self.phase -= 2.0 * PI; }
        }
    }
    fn name(&self) -> &str { "SourceNode" }
}

pub struct MixerNode;

impl AudioNode for MixerNode {
    fn process(&mut self, inputs: &[&[f32]], output: &mut [f32], _params: &NodeParams) {
        for o in output.iter_mut() { *o = 0.0; }
        for input in inputs {
            for (o, &i) in output.iter_mut().zip(input.iter()) {
                *o += i;
            }
        }
    }
    fn name(&self) -> &str { "MixerNode" }
}

pub struct GainNode {
    pub gain: f32,
}

impl GainNode {
    pub fn new(gain: f32) -> Self { GainNode { gain } }
}

impl AudioNode for GainNode {
    fn process(&mut self, inputs: &[&[f32]], output: &mut [f32], _params: &NodeParams) {
        for o in output.iter_mut() { *o = 0.0; }
        if let Some(input) = inputs.first() {
            for (o, &i) in output.iter_mut().zip(input.iter()) {
                *o = i * self.gain;
            }
        }
    }
    fn name(&self) -> &str { "GainNode" }
}

pub struct MasterNode;

impl AudioNode for MasterNode {
    fn process(&mut self, inputs: &[&[f32]], output: &mut [f32], params: &NodeParams) {
        for o in output.iter_mut() { *o = 0.0; }
        for input in inputs {
            for (o, &i) in output.iter_mut().zip(input.iter()) {
                *o += i * params.volume;
            }
        }
    }
    fn name(&self) -> &str { "MasterNode" }
}

pub struct AudioGraph {
    graph: DiGraph<Box<dyn AudioNode>, ()>,
    buffer_size: usize,
}

impl AudioGraph {
    pub fn new(buffer_size: usize) -> Self {
        AudioGraph {
            graph: DiGraph::new(),
            buffer_size,
        }
    }

    pub fn add_node(&mut self, node: Box<dyn AudioNode>) -> NodeId {
        self.graph.add_node(node)
    }

    pub fn connect(&mut self, from: NodeId, to: NodeId) -> Result<(), GraphError> {
        self.graph.add_edge(from, to, ());
        if toposort(&self.graph, None).is_err() {
            self.graph.remove_edge(self.graph.find_edge(from, to).unwrap());
            return Err(GraphError::CycleDetected);
        }
        Ok(())
    }

    pub fn process_graph(&mut self, buffer_size: usize, params: &NodeParams) -> Result<Vec<f32>, GraphError> {
        let buf_size = if buffer_size > 0 { buffer_size } else { self.buffer_size };
        let order = toposort(&self.graph, None).map_err(|_| GraphError::CycleDetected)?;
        debug!("Processing {} nodes in topological order", order.len());

        let mut outputs: std::collections::HashMap<NodeIndex, Vec<f32>> = std::collections::HashMap::new();

        let mut last_output = vec![0.0f32; buf_size];

        for node_idx in &order {
            let predecessors: Vec<NodeIndex> = self.graph
                .neighbors_directed(*node_idx, petgraph::Direction::Incoming)
                .collect();

            let input_data: Vec<Vec<f32>> = predecessors.iter()
                .filter_map(|pred| outputs.get(pred).cloned())
                .collect();

            let input_slices: Vec<&[f32]> = input_data.iter().map(|v| v.as_slice()).collect();

            let mut output = vec![0.0f32; buf_size];

            if let Some(node) = self.graph.node_weight_mut(*node_idx) {
                node.process(&input_slices, &mut output, params);
            }

            last_output = output.clone();
            outputs.insert(*node_idx, output);
        }

        Ok(last_output)
    }
}

pub struct StereoGainNode {
    pub gain_l: f32,
    pub gain_r: f32,
}

impl StereoGainNode {
    pub fn new(gain_l: f32, gain_r: f32) -> Self {
        StereoGainNode { gain_l, gain_r }
    }
}

impl AudioNode for StereoGainNode {
    fn process(&mut self, inputs: &[&[f32]], output: &mut [f32], _params: &NodeParams) {
        for o in output.iter_mut() { *o = 0.0; }
        if let Some(input) = inputs.first() {
            let len = output.len().min(input.len());
            output[..len].copy_from_slice(&input[..len]);
        }
        // Apply stereo gain to interleaved buffer
        for chunk in output.chunks_mut(2) {
            if chunk.len() == 2 {
                chunk[0] *= self.gain_l;
                chunk[1] *= self.gain_r;
            }
        }
    }
    fn name(&self) -> &str { "StereoGainNode" }
}
