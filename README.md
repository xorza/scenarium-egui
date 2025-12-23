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
