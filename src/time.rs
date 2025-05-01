use std::time::SystemTime;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub(crate) fn now() -> SystemTime {
    fn to_std_systemtime(time: web_time::SystemTime) -> std::time::SystemTime {
        let duration = time
            .duration_since(web_time::SystemTime::UNIX_EPOCH)
            .unwrap();
        SystemTime::UNIX_EPOCH + duration
    }

    // FIXME: change to:
    // #[allow(
    //     clippy::disallowed_methods,
    //     reason = "`web-time` aliases `std::time::SystemTime::now` on non-WASM platforms"
    // )]
    #[allow(clippy::disallowed_methods)]
    to_std_systemtime(web_time::SystemTime::now())
}

#[cfg(not(all(feature = "web", target_arch = "wasm32")))]
pub(crate) fn now() -> SystemTime {
    // FIXME: change to #[expect(clippy::disallowed_methods, reason = "the `web` feature is disabled")]
    #[allow(clippy::disallowed_methods)]
    SystemTime::now()
}
