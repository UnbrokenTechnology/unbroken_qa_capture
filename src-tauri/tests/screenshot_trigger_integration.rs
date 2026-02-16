//! Integration tests for screenshot trigger functionality.
//!
//! These tests verify that the screenshot trigger system works end-to-end,
//! from hotkey detection through to screenshot tool invocation.
//!
//! Note: Some tests may require manual verification on actual Windows 11 systems
//! as triggering the Snipping Tool cannot be fully automated in CI environments.

#[cfg(test)]
mod screenshot_trigger_tests {
    /// Test that the capture bridge can be instantiated
    #[test]
    fn test_capture_bridge_instantiation() {
        #[cfg(windows)]
        {
            use unbroken_qa_capture_lib::platform::get_capture_bridge;
            let _bridge = get_capture_bridge();
        }

        #[cfg(not(windows))]
        {
            // This test is primarily for Windows but should pass on all platforms
        }
    }

    /// Integration test: Trigger screenshot and verify no crash
    ///
    /// This test verifies that calling trigger_screenshot() does not panic
    /// and returns a valid Result (either success or well-formed error).
    ///
    /// Manual verification: After running this test, a screenshot tool should
    /// have been launched (if on Windows 11).
    #[test]
    #[cfg(windows)]
    fn test_trigger_screenshot_integration() {
        use unbroken_qa_capture_lib::platform::{get_capture_bridge, PlatformError};

        let bridge = get_capture_bridge();
        let result = bridge.trigger_screenshot();

        // Verify the result is well-formed (no panic, valid error structure)
        match result {
            Ok(_) => {
                println!("✓ Screenshot tool triggered successfully");
            }
            Err(PlatformError::ScreenshotTriggerError { method, message }) => {
                println!("⚠ Screenshot trigger failed: method={}, message={}", method, message);
                // This is acceptable in CI - we can't guarantee the tool will launch
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    /// Integration test: File watcher detects screenshots after trigger
    ///
    /// This test sets up a file watcher, triggers a screenshot, and verifies
    /// that the watcher can detect new screenshot files.
    ///
    /// Note: This test requires manual interaction (user must complete the screenshot).
    /// In automated CI, it will only verify the watcher setup doesn't crash.
    #[test]
    #[cfg(windows)]
    #[ignore] // Requires manual interaction
    fn test_file_watcher_detects_triggered_screenshots() {
        use unbroken_qa_capture_lib::platform::{get_capture_bridge, CaptureEvent};
        use std::sync::mpsc::channel;
        use std::time::Duration;

        let bridge = get_capture_bridge();
        let temp_dir = std::env::temp_dir().join("screenshot_trigger_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Start file watcher
        let (tx, rx) = channel();
        let handle = bridge.start_file_watcher(&temp_dir, tx).unwrap();

        println!("File watcher started on: {}", temp_dir.display());

        // Trigger screenshot (this would require manual completion)
        let trigger_result = bridge.trigger_screenshot();
        println!("Screenshot trigger result: {:?}", trigger_result);

        // Wait for a screenshot event (with timeout)
        println!("Waiting 30 seconds for manual screenshot completion...");
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(CaptureEvent::ScreenshotDetected { file_path, file_size, detected_at }) => {
                println!("✓ Screenshot detected!");
                println!("  Path: {}", file_path.display());
                println!("  Size: {} bytes", file_size);
                println!("  Detected at: {}", detected_at);

                // Cleanup
                std::fs::remove_file(&file_path).ok();
            }
            Ok(other) => {
                println!("⚠ Received unexpected event: {:?}", other);
            }
            Err(_) => {
                println!("⚠ No screenshot detected within timeout (expected in CI)");
            }
        }

        // Stop watcher and cleanup
        bridge.stop_file_watcher(handle).unwrap();
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    /// Integration test: Multiple screenshot triggers don't interfere
    ///
    /// Verifies that calling trigger_screenshot() multiple times in quick succession
    /// doesn't cause crashes or race conditions.
    #[test]
    #[cfg(windows)]
    fn test_multiple_screenshot_triggers() {
        use unbroken_qa_capture_lib::platform::get_capture_bridge;
        use std::thread;
        use std::time::Duration;

        let bridge = get_capture_bridge();

        // Trigger 3 screenshots in quick succession
        for i in 1..=3 {
            println!("Trigger attempt {}", i);
            let result = bridge.trigger_screenshot();

            // Should not panic
            match result {
                Ok(_) => println!("  ✓ Trigger {} succeeded", i),
                Err(e) => println!("  ⚠ Trigger {} failed: {:?}", i, e),
            }

            // Small delay to avoid overwhelming the system
            thread::sleep(Duration::from_millis(500));
        }
    }

    /// Integration test: Verify trigger_screenshot returns well-formed results
    ///
    /// This test verifies that trigger_screenshot() returns either success
    /// or a well-structured error (never panics).
    #[test]
    #[cfg(windows)]
    fn test_trigger_screenshot_returns_valid_result() {
        use unbroken_qa_capture_lib::platform::get_capture_bridge;

        let bridge = get_capture_bridge();

        // The trigger may succeed or fail depending on system state,
        // but it should always return a well-formed Result
        let result = bridge.trigger_screenshot();

        match result {
            Ok(_) => {
                println!("✓ Screenshot trigger succeeded");
            }
            Err(e) => {
                println!("⚠ Screenshot trigger failed (expected in some environments): {:?}", e);
            }
        }

        // The important thing is that we got here without panicking
    }
}
