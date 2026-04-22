use crate::Error;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "vzglyd_host")]
unsafe extern "C" {
    #[link_name = "channel_push"]
    fn host_channel_push(ptr: *const u8, len: i32) -> i32;
    #[link_name = "channel_poll"]
    fn host_channel_poll(ptr: *mut u8, len: i32) -> i32;
    #[link_name = "channel_active"]
    fn host_channel_active() -> i32;
    #[link_name = "log_info"]
    fn host_log_info(ptr: *const u8, len: i32) -> i32;
    #[link_name = "network_request"]
    fn host_network_request(ptr: *const u8, len: i32) -> i32;
    #[link_name = "network_response_len"]
    fn host_network_response_len() -> i32;
    #[link_name = "network_response_read"]
    fn host_network_response_read(ptr: *mut u8, len: i32) -> i32;
    /// Return the byte length of the current host-provided sidecar params JSON.
    #[link_name = "params_len"]
    fn host_params_len() -> i32;
    /// Read the current host-provided sidecar params JSON into guest memory.
    #[link_name = "params_read"]
    fn host_params_read(ptr: *mut u8, len: i32) -> i32;
    /// Sleep in the host so the host can interrupt the wait for force-refresh.
    #[link_name = "sleep_ms"]
    fn host_sleep_ms(duration_ms: i64) -> i32;
    /// Announce to the host that the sidecar is about to sleep for `duration_ms` milliseconds.
    /// Enables the TUI countdown timer. Returns 0 on success.
    #[link_name = "announce_sleep"]
    fn host_announce_sleep(duration_ms: i64) -> i32;
    /// Publish a named artifact. `kind` is a UTF-8 string (e.g. `"raw_source_payload"`).
    /// Returns 0 on success.
    #[link_name = "artifact_publish"]
    fn host_artifact_publish(
        kind_ptr: *const u8,
        kind_len: i32,
        data_ptr: *const u8,
        data_len: i32,
    ) -> i32;
    /// Register the sidecar manifest with the host. Call once at startup.
    /// Returns 0 on success.
    #[link_name = "register_manifest"]
    fn host_register_manifest(ptr: *const u8, len: i32) -> i32;
}

/// Push a new payload into the shared sidecar-to-slide channel.
///
/// Returns the raw host status code (0 = success). On non-WASM targets it always succeeds (0).
pub fn channel_push(data: &[u8]) -> i32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return host_channel_push(data.as_ptr(), data.len() as i32);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = data;
        0
    }
}

/// Poll the shared channel for the latest payload.
///
/// The host writes into `buf` and returns the number of bytes copied. A negative return value
/// indicates that no new payload was available or that the buffer was too small.
pub fn channel_poll(buf: &mut [u8]) -> i32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return host_channel_poll(buf.as_mut_ptr(), buf.len() as i32);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = buf;
        0
    }
}

/// Return `true` when the paired slide is currently active on screen.
pub fn channel_active() -> bool {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return host_channel_active() != 0;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// Sleep for a whole number of seconds.
pub fn sleep_secs(secs: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        let _ = host_sleep_ms(i64::from(secs) * 1000);
    }

    #[cfg(not(target_arch = "wasm32"))]
    std::thread::sleep(std::time::Duration::from_secs(u64::from(secs)));
}

/// Read the current sidecar params JSON supplied by the host.
///
/// Returns `Ok(None)` when the host has no params for this sidecar. On WASM targets, the value can
/// change between poll iterations, so sidecars should call this near the start of each fetch cycle
/// when they want live-editable configuration.
///
/// # Errors
///
/// Returns [`Error`] if the host reports an invalid params length or the params buffer cannot be
/// read.
pub fn runtime_params_bytes() -> Result<Option<Vec<u8>>, Error> {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        let len = host_params_len();
        if len < 0 {
            return Err(Error::Io(format!(
                "host params_len failed with status {len}"
            )));
        }
        if len == 0 {
            return Ok(None);
        }
        let mut bytes = vec![0u8; len as usize];
        let read = host_params_read(bytes.as_mut_ptr(), bytes.len() as i32);
        if read < 0 {
            return Err(Error::Io(format!(
                "host params_read failed with status {read}"
            )));
        }
        bytes.truncate(read as usize);
        return Ok(Some(bytes));
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(None)
    }
}

/// Emit an informational log message through the VZGLYD host.
pub fn info_log(message: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        let _ = host_log_info(message.as_ptr(), message.len() as i32);
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = message;
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn network_roundtrip(request: &[u8]) -> Result<Vec<u8>, Error> {
    unsafe {
        let submit_status = host_network_request(request.as_ptr(), request.len() as i32);
        if submit_status != 0 {
            return Err(Error::Io(format!(
                "host network_request failed with status {submit_status}"
            )));
        }

        let response_len = host_network_response_len();
        if response_len < 0 {
            return Err(Error::Io(format!(
                "host network_response_len failed with status {response_len}"
            )));
        }

        let mut response = vec![0u8; response_len as usize];
        let read_status = host_network_response_read(response.as_mut_ptr(), response.len() as i32);
        if read_status < 0 {
            return Err(Error::Io(format!(
                "host network_response_read failed with status {read_status}"
            )));
        }

        response.truncate(read_status as usize);
        return Ok(response);
    }
}

/// Announce to the host that the sidecar is about to sleep for `duration_ms` milliseconds.
///
/// The brrmmmm TUI uses this to display a countdown timer. Call this immediately before
/// any `sleep_secs` call. On non-WASM targets this is a no-op.
///
/// Returns the host status code (0 = success).
pub fn announce_sleep(duration_ms: i64) -> i32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return host_announce_sleep(duration_ms);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = duration_ms;
        0
    }
}

/// Publish a named artifact to the brrmmmm runtime.
///
/// This is the v2 alternative to [`channel_push`], which allows the runtime to distinguish
/// between `raw_source_payload`, `normalized_payload`, and `published_output`.
///
/// On non-WASM targets this is a no-op.
///
/// Returns 0 on success.
pub fn artifact_publish(kind: &str, data: &[u8]) -> i32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return host_artifact_publish(
            kind.as_ptr(),
            kind.len() as i32,
            data.as_ptr(),
            data.len() as i32,
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (kind, data);
        0
    }
}

/// Convenience: publish raw source payload (the unprocessed HTTP response body).
pub fn publish_raw(data: &[u8]) -> i32 {
    artifact_publish("raw_source_payload", data)
}

/// Convenience: publish normalized payload (parsed/transformed).
pub fn publish_normalized(data: &[u8]) -> i32 {
    artifact_publish("normalized_payload", data)
}

/// Convenience: publish final output (equivalent to `channel_push`).
pub fn publish_output(data: &[u8]) -> i32 {
    artifact_publish("published_output", data)
}

/// Register the sidecar manifest with the brrmmmm runtime.
///
/// Call once at the very beginning of `main`, before [`crate::poll_loop`].
/// This populates the TUI with env var requirements, polling strategy, and
/// the sidecar's behavioral contract.
///
/// On non-WASM targets this is a no-op.
///
/// Returns 0 on success.
pub fn register_manifest(manifest: &crate::manifest::SidecarManifest) -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        let bytes = match serde_json::to_vec(manifest) {
            Ok(b) => b,
            Err(_) => return -1,
        };
        unsafe {
            return host_register_manifest(bytes.as_ptr(), bytes.len() as i32);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = manifest;
        0
    }
}
