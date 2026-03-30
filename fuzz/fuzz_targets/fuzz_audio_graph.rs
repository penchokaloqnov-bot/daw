#![no_main]
use libfuzzer_sys::fuzz_target;
use daw_engine::{AudioGraph, NodeParams, SourceNode, GainNode, MixerNode};

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 { return; }
    let num_nodes = (data[0] % 8 + 1) as usize;
    let buffer_size = 128;
    let mut graph = AudioGraph::new(buffer_size);
    let mut nodes = Vec::new();
    for i in 0..num_nodes {
        let freq = 110.0 + (data[i % data.len()] as f32) * 10.0;
        let node = Box::new(SourceNode::new(freq, 44100.0));
        nodes.push(graph.add_node(node));
    }
    // Try to add edges (graph validates no cycles)
    for i in 1..nodes.len().min(data.len() / 2) {
        let from = nodes[(data[i] as usize) % i];
        let to = nodes[i];
        let _ = graph.connect(from, to); // ignore errors
    }
    let params = NodeParams::default();
    let _ = graph.process_graph(buffer_size, &params);
});
