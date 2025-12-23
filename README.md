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

The egui app renders the sample graph via `src/gui/graph.rs`, which draws a transparent dotted background before delegating to `src/gui/node.rs` for node rectangles, labels, text, and cubic bezier connection curves. Nodes can be dragged by their title bar to reposition them, and the graph view supports panning and zoom gestures stored on the graph itself (including scaled text and connection curvature). Inputs and outputs are rendered with larger circular ports that stick out from the node sides; inputs are blue, outputs are blue-green, and each port brightens on hover. Panning only triggers when the pointer is over empty space. Clicking or dragging a node selects it and draws a brighter border using `Graph::selected_node_id`. To break connections, click and drag on empty space to paint a red breaker stroke (up to a fixed length); intersected connections turn red and are removed when you release the mouse button. To create connections, click and drag from a port to a nearby port of the opposite type; releasing within the activation radius connects the ports and overwrites any existing input connection. Temporary connection drags now mirror the port direction, so starting from an input bends the curve left before heading to the target.

Startup initialization (dotenv + tracing) lives in `src/init.rs`.

## Main menu

The top menu bar exposes a **File** menu with:

- **New**: resets to an empty graph (`Graph::default`).
- **Save**: serializes the current graph to a JSON file in the system temp directory (`scenarium-graph.json`).
- **Load**: deserializes the graph from that JSON file and replaces the current graph.
- **Test**: loads the sample graph from `Graph::test_graph`.

The app displays a short status message after each command, and the menu uses larger text with wider padding for easier clicking.

## Graph serialization

`Graph` serializes and deserializes directly (via serde) using `GraphFormat::{Toml, Yaml, Json}` in `src/model.rs`. The API validates loaded graphs and expects internal graphs to be valid before serialization. File helpers (`serialize_to_file`/`deserialize_from_file`) choose the format based on the file extension (`.toml`, `.yaml`/`.yml`, `.json`).

`Graph::default` returns an empty graph with a new UUID, zero pan, and a zoom of `1.0`.

## WGPU backend features

The app uses the eframe WGPU renderer, and backend features are enabled via a direct `wgpu` dependency pinned to the same version as eframe (currently `27.0.1`). If you upgrade eframe, update the `wgpu` version and backend feature list (`metal`, `vulkan`, `dx12`) to match so at least one native backend is compiled; otherwise startup can panic with "No wgpu backend feature that is implemented for the target platform was enabled."
