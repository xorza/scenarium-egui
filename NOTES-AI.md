# NOTES-AI

This file is AI-generated and contains implementation details, project structure, and functionality notes for fast AI access. Update this file (not `README.md`) when adding or changing implementation details.

## Project Structure

- `src/`
  - `main.rs`: application entry point and egui app wiring.
  - `init.rs`: startup initialization (dotenv + tracing).
  - `model.rs`: data model, serialization, validation, and test graph builder.
- `gui/`
  - `graph.rs`: graph view input handling, background, connections, and overall rendering orchestration.
  - `node.rs`: node geometry, ports, label layout, and node body interactions.
- `render.rs`: shared `RenderContext` + `WidgetRenderer` trait for reusable rendering helpers.
- `style.rs`: centralized UI styling constants (colors, padding factors, stroke styles).
    - `mod.rs`: gui module exports.
- `assets/`: window icon, fonts, and Wayland desktop entry.

## Functionality

### GUIDs (UUIDs)
- Uses `uuid` crate to generate UUID v4 values.
- UI includes a button to generate and render a GUID.

### Graph Data Model
- `Graph::test_graph` builds a sample node graph:
  - `value_a` (one output)
  - `value_b` (one output)
  - `math(sum)` with inputs from value_a/value_b, output named `sum`
  - `math(divide)` with inputs from sum/value_b, output named `divide`
  - `output` node connected to divide
- `Graph::validate` enforces:
  - finite/positive zoom, finite pan and node positions
  - unique node IDs
  - selected node exists
  - input connections reference existing nodes and output indices
- `Graph::remove_node` removes a node, clears selection if needed, and nulls inbound connections referencing the removed node.

### Graph Rendering + Interaction

#### Rendering Pipeline
- `graph.rs` orchestrates rendering with a shared `RenderContext`:
  - background (dotted grid)
  - connections (including breaker highlights)
  - node bodies
  - ports
  - labels
- Shared rendering utilities live in `render.rs` with:
  - `RenderContext`: per-frame painter/layout/fonts/widths
  - `WidgetRenderer` trait for small rendering components

#### Node Layout + Sizing
- `NodeLayout` defines base node dimensions and padding.
- Node widths auto-size based on the widest label (title/inputs/outputs) with a minimum base width.
- No extra inter-column padding between input/output labels (to keep nodes tighter).

#### Node Widgets
- Node title bar supports drag-to-move.
- Node body and title bar support selection.
- Each node has a small `x` button in the top-right title bar:
  - hover tooltip: “Remove node”
  - pressed/hover styling
  - removing a node clears inbound connections
- Each node has a small panel under the title with a compact `cache` button (turns yellow when active) that toggles `Node::cache_output`.

#### Ports + Connections
- Inputs/outputs are rendered as circular ports; hover brightens color.
- Port positions are computed per node width and layout.
- Connection curves are cubic Beziers using a control offset derived from horizontal distance.
- Dragging from a port shows a temporary connection curve.

#### Panning + Zooming
- Dragging empty space pans the graph.
- Middle mouse drag also pans.
- Touchpad/scroll wheel pans when cursor is over the graph.
- Pinch-to-zoom (trackpad) or Ctrl/Cmd + scroll zooms, centered on cursor.

#### Breaker Tool
- Dragging empty space draws a red breaker stroke (length limited).
- Intersected connections highlight and are removed on release.

### Menus + UI
- **File** menu:
  - **New**: reset to empty graph
  - **Save**: serialize graph to temp JSON (`scenarium-graph.json`)
  - **Load**: deserialize and replace
  - **Test**: load `Graph::test_graph`
- Menu uses larger text and padding; short status messages displayed after actions.

### Serialization
- `Graph` serializes/deserializes with `serde` via `GraphFormat::{Toml, Yaml, Json}`.
- File helpers choose format by file extension.
- `Graph::default` yields empty graph, new UUID, zero pan, zoom = 1.0.

### Assets + System Integration
- Window icon: `assets/icon.png`.
- Wayland: app ID `scenarium-egui` + sample desktop entry at `assets/scenarium-egui.desktop`.
- Fonts: bundled Raleway SemiBold at `assets/Raleway/static/Raleway-SemiBold.ttf`.
- Text color: global brighter tint applied to labels.

### WGPU
- WGPU backend features are pinned to eframe version; keep `wgpu` version and features (`metal`, `vulkan`, `dx12`) aligned with eframe to avoid missing-backend panics.
