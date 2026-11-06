//! Bounds temporary Cocoa allocations made by long-lived Rust workers.

pub(crate) fn run<T>(operation: impl FnOnce() -> T) -> T {
    // Rust worker threads do not have AppKit's periodically drained pool.
    // Retained native values may escape; autoreleased temporaries must not.
    #[cfg(target_os = "macos")]
    {
        objc2::rc::autoreleasepool(|_| operation())
    }
    #[cfg(not(target_os = "macos"))]
    {
        operation()
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use objc2::{
        rc::{autoreleasepool_leaking, Retained, Weak},
        runtime::NSObject,
    };

    #[test]
    fn worker_refreshes_release_native_temporaries_before_the_thread_exits() {
        std::thread::spawn(|| {
            for _ in 0..1_000 {
                let weak = super::run(|| {
                    let object = NSObject::new();
                    let weak = Weak::from_retained(&object);
                    // Model a Cocoa getter that returns an autoreleased value
                    // to its caller's pool, without introducing a nested pool.
                    autoreleasepool_leaking(|pool| {
                        // SAFETY: `run` owns the active pool on this thread;
                        // no reference to this autoreleased object escapes it.
                        let _ = unsafe { Retained::autorelease(object, pool) };
                    });
                    assert!(weak.load().is_some());
                    weak
                });
                assert!(
                    weak.load().is_none(),
                    "refresh kept its native temporary alive"
                );
            }
        })
        .join()
        .expect("native worker memory check failed");
    }
}
