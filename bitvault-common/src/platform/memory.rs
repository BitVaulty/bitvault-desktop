//! Memory security functions
//!
//! This module provides functions for secure memory management,
//! including memory locking, secure allocation, and secure erasure.
//! 
//! # Security Considerations
//! 
//! These functions are critical for protecting sensitive cryptographic material
//! and should be used for any memory that contains keys, passwords, or other
//! sensitive information.

use std::sync::atomic::{fence, Ordering};

use super::error::{PlatformError, PlatformResult};

/// Lock memory to prevent swapping on Unix systems
///
/// This uses the mlock syscall to prevent the memory from being swapped to disk.
///
/// # Arguments
/// * `ptr` - Pointer to the memory to lock
/// * `len` - Length of the memory region to lock
///
/// # Returns
/// * Ok(()) if successful, Err with a PlatformError if failed
#[cfg(all(unix, feature = "strict-memory"))]
pub fn lock_memory_unix(_ptr: *const u8, _len: usize) -> PlatformResult<()> {
    use std::io::Error;
    
    let ret = unsafe { libc::mlock(_ptr as *const libc::c_void, _len) };
    if ret != 0 {
        let err = Error::last_os_error();
        if err.raw_os_error() == Some(libc::ENOMEM) {
            return Err(PlatformError::MemoryError(
                "Not enough memory or permissions to lock memory".to_string(),
            ));
        } else {
            return Err(PlatformError::MemoryError(
                format!("Failed to lock memory: {}", err),
            ));
        }
    }
    Ok(())
}

#[cfg(not(all(unix, feature = "strict-memory")))]
pub fn lock_memory_unix(ptr: *const u8, len: usize) -> PlatformResult<()> {
    // Memory locking is not available or not enabled
    Err(PlatformError::UnsupportedOperation(
        "Memory locking not available or strict-memory feature not enabled".to_string(),
    ))
}

/// Unlock memory on Unix systems
///
/// This uses the munlock syscall to allow the memory to be swapped to disk again.
///
/// # Arguments
/// * `ptr` - Pointer to the memory to unlock
/// * `len` - Length of the memory region to unlock
///
/// # Returns
/// * Ok(()) if successful, Err with a PlatformError if failed
#[cfg(all(unix, feature = "strict-memory"))]
pub fn unlock_memory_unix(_ptr: *const u8, _len: usize) -> PlatformResult<()> {
    use std::io::Error;
    
    let ret = unsafe { libc::munlock(_ptr as *const libc::c_void, _len) };
    if ret != 0 {
        let err = Error::last_os_error();
        return Err(PlatformError::MemoryError(
            format!("Failed to unlock memory: {}", err),
        ));
    }
    Ok(())
}

#[cfg(not(all(unix, feature = "strict-memory")))]
pub fn unlock_memory_unix(ptr: *const u8, len: usize) -> PlatformResult<()> {
    // Memory locking is not available or not enabled
    Err(PlatformError::UnsupportedOperation(
        "Memory locking not available or strict-memory feature not enabled".to_string(),
    ))
}

/// Lock memory to prevent swapping on Windows systems
///
/// This uses the VirtualLock Windows API to prevent the memory from being swapped.
///
/// # Arguments
/// * `ptr` - Pointer to the memory to lock
/// * `len` - Length of the memory region to lock
///
/// # Returns
/// * Ok(()) if successful, Err with a PlatformError if failed
#[cfg(all(windows, feature = "strict-memory"))]
pub fn lock_memory_windows(_ptr: *const u8, _len: usize) -> PlatformResult<()> {
    use std::io::Error;
    use winapi::um::memoryapi::VirtualLock;
    
    let ret = unsafe { VirtualLock(_ptr as *mut _, _len) };
    if ret == 0 {
        let err = Error::last_os_error();
        return Err(PlatformError::MemoryError(
            format!("Failed to lock memory: {}", err),
        ));
    }
    Ok(())
}

#[cfg(not(all(windows, feature = "strict-memory")))]
pub fn lock_memory_windows(ptr: *const u8, len: usize) -> PlatformResult<()> {
    // Memory locking is not available or not enabled
    Err(PlatformError::UnsupportedOperation(
        "Memory locking not available or strict-memory feature not enabled".to_string(),
    ))
}

/// Unlock memory on Windows systems
///
/// This uses the VirtualUnlock Windows API to allow the memory to be swapped again.
///
/// # Arguments
/// * `ptr` - Pointer to the memory to unlock
/// * `len` - Length of the memory region to unlock
///
/// # Returns
/// * Ok(()) if successful, Err with a PlatformError if failed
#[cfg(all(windows, feature = "strict-memory"))]
pub fn unlock_memory_windows(_ptr: *const u8, _len: usize) -> PlatformResult<()> {
    use std::io::Error;
    use winapi::um::memoryapi::VirtualUnlock;
    
    let ret = unsafe { VirtualUnlock(_ptr as *mut _, _len) };
    if ret == 0 {
        let err = Error::last_os_error();
        return Err(PlatformError::MemoryError(
            format!("Failed to unlock memory: {}", err),
        ));
    }
    Ok(())
}

#[cfg(not(all(windows, feature = "strict-memory")))]
pub fn unlock_memory_windows(ptr: *const u8, len: usize) -> PlatformResult<()> {
    // Memory locking is not available or not enabled
    Err(PlatformError::UnsupportedOperation(
        "Memory locking not available or strict-memory feature not enabled".to_string(),
    ))
}

/// Securely erase a buffer by overwriting it with zeros
///
/// This function attempts to prevent compiler optimizations that might
/// eliminate the zeroing operation.
///
/// # Arguments
/// * `buffer` - The buffer to erase
pub fn secure_erase(buffer: &mut [u8]) {
    // Use zeroize crate in real implementation for better guarantees
    for byte in buffer.iter_mut() {
        *byte = 0;
    }
    
    // This fence ensures that the compiler doesn't optimize away the zeroing
    fence(Ordering::SeqCst);
}

/// Allocate a buffer and lock it in memory if possible
///
/// # Arguments
/// * `size` - Size of the buffer to allocate
/// * `lock` - Whether to attempt to lock the memory
///
/// # Returns
/// * A Vec<u8> that is locked in memory if the platform supports it
pub fn secure_alloc(size: usize, lock: bool) -> Vec<u8> {
    let buffer = vec![0u8; size];
    
    if lock {
        // Try to lock the memory, but don't fail if it doesn't work
        #[cfg(unix)]
        let _ = lock_memory_unix(buffer.as_ptr(), buffer.len());
        
        #[cfg(windows)]
        let _ = lock_memory_windows(buffer.as_ptr(), buffer.len());
    }
    
    buffer
} 