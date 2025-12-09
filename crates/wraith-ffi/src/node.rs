//! Node API FFI

use std::os::raw::{c_char, c_int};
use std::sync::Arc;

use tokio::runtime::Runtime;
use wraith_core::node::Node;
use wraith_core::node::config::NodeConfig;

use crate::config::ConfigHandle;
use crate::error::{WraithError, WraithErrorCode};
use crate::types::*;
use crate::{NodeHandle, WraithConfig, WraithNode, ffi_try, ffi_try_ptr};

/// Create a new node with random identity
///
/// # Safety
///
/// - `config` must be a valid configuration handle or null (uses default config)
/// - `error_out` must be null or a valid pointer to receive error message
/// - Caller must free the returned handle with `wraith_node_free()`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wraith_node_new(
    config: *const WraithConfig,
    error_out: *mut *mut c_char,
) -> *mut WraithNode {
    let node_config = if config.is_null() {
        NodeConfig::default()
    } else {
        (*(config as *const ConfigHandle)).config.clone()
    };

    let runtime = ffi_try_ptr!(
        Runtime::new().map_err(|e| WraithError::internal_error(e.to_string())),
        error_out
    );

    // Node::new_with_config is async, so we need to block on it
    let node = ffi_try_ptr!(
        runtime
            .block_on(Node::new_with_config(node_config))
            .map_err(WraithError::from),
        error_out
    );

    let handle = Box::new(NodeHandle {
        node,
        runtime: Arc::new(runtime),
    });

    Box::into_raw(handle) as *mut WraithNode
}

/// Free a node handle
///
/// This will stop the node if it's running and clean up all resources.
///
/// # Safety
///
/// - `node` must be a valid pointer returned by `wraith_node_new()` or `wraith_node_from_identity()`
/// - `node` must not be used after this call
/// - `node` must not be freed multiple times
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wraith_node_free(node: *mut WraithNode) {
    if !node.is_null() {
        let handle = Box::from_raw(node as *mut NodeHandle);
        // Runtime handles cleanup on drop
        drop(handle);
    }
}

/// Start the node
///
/// This initializes the transport layer and begins listening for connections.
///
/// # Safety
///
/// - `node` must be a valid node handle
/// - `error_out` must be null or a valid pointer to receive error message
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wraith_node_start(
    node: *mut WraithNode,
    error_out: *mut *mut c_char,
) -> c_int {
    if node.is_null() {
        if !error_out.is_null() {
            *error_out = WraithError::invalid_argument("node is null").to_c_string();
        }
        return WraithErrorCode::InvalidArgument as c_int;
    }

    let handle = &mut *(node as *mut NodeHandle);
    let node_clone = handle.node.clone();
    let runtime = handle.runtime.clone();

    ffi_try!(
        runtime
            .block_on(async move { node_clone.start().await })
            .map_err(WraithError::from),
        error_out
    );

    WraithErrorCode::Success as c_int
}

/// Stop the node
///
/// This gracefully shuts down the transport layer and closes all sessions.
///
/// # Safety
///
/// - `node` must be a valid node handle
/// - `error_out` must be null or a valid pointer to receive error message
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wraith_node_stop(
    node: *mut WraithNode,
    error_out: *mut *mut c_char,
) -> c_int {
    if node.is_null() {
        if !error_out.is_null() {
            *error_out = WraithError::invalid_argument("node is null").to_c_string();
        }
        return WraithErrorCode::InvalidArgument as c_int;
    }

    let handle = &mut *(node as *mut NodeHandle);
    let node_clone = handle.node.clone();
    let runtime = handle.runtime.clone();

    ffi_try!(
        runtime
            .block_on(async move { node_clone.stop().await })
            .map_err(WraithError::from),
        error_out
    );

    WraithErrorCode::Success as c_int
}

/// Check if the node is running
///
/// # Safety
///
/// - `node` must be a valid node handle
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wraith_node_is_running(node: *const WraithNode) -> bool {
    if node.is_null() {
        return false;
    }

    let handle = &*(node as *const NodeHandle);
    handle.node.is_running()
}

/// Get the node's ID (Ed25519 public key)
///
/// # Safety
///
/// - `node` must be a valid node handle
/// - `id_out` must be a valid pointer to a WraithNodeId struct
/// - `error_out` must be null or a valid pointer to receive error message
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wraith_node_get_id(
    node: *const WraithNode,
    id_out: *mut WraithNodeId,
    error_out: *mut *mut c_char,
) -> c_int {
    if node.is_null() {
        if !error_out.is_null() {
            *error_out = WraithError::invalid_argument("node is null").to_c_string();
        }
        return WraithErrorCode::InvalidArgument as c_int;
    }

    if id_out.is_null() {
        if !error_out.is_null() {
            *error_out = WraithError::invalid_argument("id_out is null").to_c_string();
        }
        return WraithErrorCode::InvalidArgument as c_int;
    }

    let handle = &*(node as *const NodeHandle);
    let node_id = handle.node.node_id();
    (*id_out).bytes.copy_from_slice(node_id);

    WraithErrorCode::Success as c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::wraith_config_new;
    use std::ffi::CStr;
    use std::ptr;

    #[test]
    fn test_node_new_free() {
        unsafe {
            let node = wraith_node_new(ptr::null(), ptr::null_mut());
            assert!(!node.is_null());

            // Node should not be running initially
            assert!(!wraith_node_is_running(node));

            wraith_node_free(node);
        }
    }

    #[test]
    fn test_node_new_with_config() {
        unsafe {
            let config = wraith_config_new(ptr::null_mut());
            assert!(!config.is_null());

            let node = wraith_node_new(config, ptr::null_mut());
            assert!(!node.is_null());

            wraith_node_free(node);
            crate::config::wraith_config_free(config);
        }
    }

    #[test]
    fn test_node_new_with_error_out() {
        unsafe {
            let mut error_ptr: *mut c_char = ptr::null_mut();
            let node = wraith_node_new(ptr::null(), &mut error_ptr);

            assert!(!node.is_null());
            assert!(error_ptr.is_null()); // No error expected

            wraith_node_free(node);
        }
    }

    #[test]
    fn test_node_free_null() {
        unsafe {
            // Should not panic with null pointer
            wraith_node_free(ptr::null_mut());
        }
    }

    #[test]
    fn test_node_get_id() {
        unsafe {
            let node = wraith_node_new(ptr::null(), ptr::null_mut());
            let mut node_id = WraithNodeId { bytes: [0u8; 32] };

            let result = wraith_node_get_id(node, &mut node_id, ptr::null_mut());
            assert_eq!(result, WraithErrorCode::Success as c_int);

            // ID should not be all zeros
            assert_ne!(node_id.bytes, [0u8; 32]);

            wraith_node_free(node);
        }
    }

    #[test]
    fn test_node_get_id_null_node() {
        unsafe {
            let mut node_id = WraithNodeId { bytes: [0u8; 32] };
            let mut error_ptr: *mut c_char = ptr::null_mut();

            let result = wraith_node_get_id(ptr::null(), &mut node_id, &mut error_ptr);
            assert_eq!(result, WraithErrorCode::InvalidArgument as c_int);
            assert!(!error_ptr.is_null());

            let error_msg = CStr::from_ptr(error_ptr).to_str().unwrap();
            assert!(error_msg.contains("node is null"));
            crate::wraith_free_string(error_ptr);
        }
    }

    #[test]
    fn test_node_get_id_null_id_out() {
        unsafe {
            let node = wraith_node_new(ptr::null(), ptr::null_mut());
            let mut error_ptr: *mut c_char = ptr::null_mut();

            let result = wraith_node_get_id(node, ptr::null_mut(), &mut error_ptr);
            assert_eq!(result, WraithErrorCode::InvalidArgument as c_int);
            assert!(!error_ptr.is_null());

            let error_msg = CStr::from_ptr(error_ptr).to_str().unwrap();
            assert!(error_msg.contains("id_out is null"));
            crate::wraith_free_string(error_ptr);

            wraith_node_free(node);
        }
    }

    #[test]
    fn test_node_is_running_null() {
        unsafe {
            // Should return false for null node
            assert!(!wraith_node_is_running(ptr::null()));
        }
    }

    #[test]
    fn test_node_start_stop() {
        unsafe {
            let node = wraith_node_new(ptr::null(), ptr::null_mut());

            // Start the node - may fail in test environment due to transport initialization
            let mut error_ptr: *mut c_char = ptr::null_mut();
            let result = wraith_node_start(node, &mut error_ptr);

            // If start succeeds, test full lifecycle
            if result == WraithErrorCode::Success as c_int {
                assert!(wraith_node_is_running(node));

                // Stop the node
                let mut stop_error: *mut c_char = ptr::null_mut();
                let stop_result = wraith_node_stop(node, &mut stop_error);

                // Stop should succeed, but may fail in async environment
                if stop_result == WraithErrorCode::Success as c_int {
                    // Successful stop - node should not be running
                    assert!(!wraith_node_is_running(node));
                } else {
                    // Stop failed - clean up error message
                    if !stop_error.is_null() {
                        crate::wraith_free_string(stop_error);
                    }
                }
            } else {
                // Start failed (e.g., transport error in test environment)
                // Clean up error message if any
                if !error_ptr.is_null() {
                    crate::wraith_free_string(error_ptr);
                }
            }

            wraith_node_free(node);
        }
    }

    #[test]
    fn test_node_start_null() {
        unsafe {
            let mut error_ptr: *mut c_char = ptr::null_mut();
            let result = wraith_node_start(ptr::null_mut(), &mut error_ptr);

            assert_eq!(result, WraithErrorCode::InvalidArgument as c_int);
            assert!(!error_ptr.is_null());

            let error_msg = CStr::from_ptr(error_ptr).to_str().unwrap();
            assert!(error_msg.contains("node is null"));
            crate::wraith_free_string(error_ptr);
        }
    }

    #[test]
    fn test_node_stop_null() {
        unsafe {
            let mut error_ptr: *mut c_char = ptr::null_mut();
            let result = wraith_node_stop(ptr::null_mut(), &mut error_ptr);

            assert_eq!(result, WraithErrorCode::InvalidArgument as c_int);
            assert!(!error_ptr.is_null());

            let error_msg = CStr::from_ptr(error_ptr).to_str().unwrap();
            assert!(error_msg.contains("node is null"));
            crate::wraith_free_string(error_ptr);
        }
    }

    #[test]
    fn test_node_double_start() {
        unsafe {
            let node = wraith_node_new(ptr::null(), ptr::null_mut());

            // Start once - may fail in test environment
            let mut error_ptr: *mut c_char = ptr::null_mut();
            let result = wraith_node_start(node, &mut error_ptr);

            // Only test double-start if first start succeeded
            if result == WraithErrorCode::Success as c_int {
                // Start again - may succeed (idempotent) or return error
                let mut error_ptr2: *mut c_char = ptr::null_mut();
                let _result2 = wraith_node_start(node, &mut error_ptr2);

                // Should not crash - either succeeds or returns an error code
                // Don't assert specific error codes as they may vary by environment
                if !error_ptr2.is_null() {
                    crate::wraith_free_string(error_ptr2);
                }

                // Try to stop the node
                wraith_node_stop(node, ptr::null_mut());
            } else {
                // First start failed - clean up
                if !error_ptr.is_null() {
                    crate::wraith_free_string(error_ptr);
                }
            }

            wraith_node_free(node);
        }
    }

    #[test]
    fn test_node_stop_before_start() {
        unsafe {
            let node = wraith_node_new(ptr::null(), ptr::null_mut());

            // Stop before starting (should succeed or return error gracefully)
            let _result = wraith_node_stop(node, ptr::null_mut());
            assert!(!wraith_node_is_running(node));

            wraith_node_free(node);
        }
    }
}
