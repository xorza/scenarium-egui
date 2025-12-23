# egui-playground

A small egui sandbox for experimenting with widgets and layout.

## GUIDs (UUIDs)

This project uses the `uuid` crate to generate GUIDs (UUID v4). The app includes a button that generates a new GUID and renders it in the UI.

Example usage in Rust:

```rust
use uuid::Uuid;

let guid = Uuid::new_v4();
println!("{guid}");
```

## Sample graph

`Graph::test_graph` in `src/model.rs` creates a small node graph for UI rendering:

- value_a (one output)
- value_b (one output)
- math(sum) with inputs from value_a/value_b and an output named sum
- math(divide) with inputs from sum/value_b and an output named divide
- output node connected to divide

`Graph::validate` returns a `Result` if any connection references a missing node or an invalid output index.

## Graph rendering

The egui app renders the sample graph via `src/gui/graph.rs`, which draws a transparent dotted background before delegating to `src/gui/node.rs` for node rectangles, labels, and cubic bezier connection curves. Nodes can be dragged by their title bar to reposition them.

Startup initialization (dotenv + tracing) lives in `src/init.rs`.

## WGPU backend features

The app uses the eframe WGPU renderer, and backend features are enabled via a direct `wgpu` dependency pinned to the same version as eframe (currently `27.0.1`). If you upgrade eframe, update the `wgpu` version and backend feature list (`metal`, `vulkan`, `dx12`) to match so at least one native backend is compiled; otherwise startup can panic with "No wgpu backend feature that is implemented for the target platform was enabled."
