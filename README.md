# `VRX-64-sidecar`

`VRX-64-sidecar` is the standard library for VZGLYD sidecars: small `wasm32-wasip1` programs that fetch live data and push it to a paired slide.

Add it to a sidecar crate:

```toml
[dependencies]
vzglyd_sidecar = { package = "VRX-64-sidecar", path = "../lume-sidecar" }
```

Typical usage:

```rust
use vzglyd_sidecar::{https_get_text, poll_loop};

fn main() {
    poll_loop(300, || {
        let body = https_get_text("api.example.com", "/forecast")?;
        Ok(body.into_bytes())
    });
}
```

## Tracing

Sidecars can emit guest spans and events without depending on a host-specific SDK:

```rust
use vzglyd_sidecar::{trace_event, trace_scope};

let mut scope = trace_scope("fetch");
trace_event("channel_push");
scope.set_status("ok");
```

The standard `poll_loop()` and host request path are already instrumented, so most sidecars only need slide-specific scopes around parsing or business logic.

This crate is intended for the `wasm32-wasip1` target used by VZGLYD sidecars.

Further reading:

- [Slide authoring guide](https://github.com/vzglyd/vzglyd/blob/main/docs/authoring-guide.md)
- [VRX-64-sidecar repository](https://github.com/vzglyd/VRX-64-sidecar)
- [VZGLYD repository](https://github.com/vzglyd/vzglyd)
