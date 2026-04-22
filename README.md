# `VRX-64-sidecar`

Archived for reference only. Active VRX-64 development no longer ships or runs sidecars; live data acquisition now lives outside the host runtime, with tools such as `brrmmmm` writing watched JSON result files for `data_path`.

The material below documents the retired sidecar approach.

## Historical Usage

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

Sidecars can advertise editable JSON configuration in their registered manifest.
Hosts pass those values through the same optional `vzglyd_configure` buffer used
by slides, but the values are scoped to the sidecar's fetching behavior:

```rust
use vzglyd_sidecar::{
    EnvVarSpec, PollStrategy, SidecarManifest, SidecarParamField, SidecarParamType,
    SidecarParamsSchema, register_manifest,
};

register_manifest(&SidecarManifest {
    schema_version: 1,
    logical_id: "weather".into(),
    name: "Weather".into(),
    description: "Fetches forecast data".into(),
    run_modes: vec!["managed_polling".into()],
    state_persistence: Default::default(),
    required_env_vars: vec![],
    optional_env_vars: vec![EnvVarSpec {
        name: "WEATHER_API_KEY".into(),
        description: "Fallback API key supplied by the host environment".into(),
    }],
    params: Some(SidecarParamsSchema {
        fields: vec![SidecarParamField {
            key: "api_key".into(),
            kind: SidecarParamType::String,
            required: false,
            label: Some("API key".into()),
            help: Some("Overrides the default empty API key for this playlist entry".into()),
            default: Some(serde_json::json!("")),
            options: vec![],
        }],
    }),
    capabilities_needed: vec!["https_get".into()],
    poll_strategy: Some(PollStrategy::FixedInterval { interval_secs: 300 }),
    artifact_types: vec!["published_output".into()],
});
```

## Tracing

Use `export_traced_main!` for the top-level sidecar entrypoint so every run emits a stable
`sidecar.main` guest span automatically:

```rust
fn sidecar_main() {
    poll_loop(300, || Ok(Vec::new()));
}

#[cfg(target_arch = "wasm32")]
vzglyd_sidecar::export_traced_main!("example_sidecar", sidecar_main);
```

Sidecars can also emit finer-grained guest spans and events without depending on a host-specific SDK:

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
