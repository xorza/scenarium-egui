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
