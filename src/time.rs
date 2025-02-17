use std::time::SystemTime;

#[cfg(feature = "web")]
pub(crate) fn now() -> SystemTime {
    fn to_std_systemtime(time: web_time::SystemTime) -> std::time::SystemTime {
        let duration = time
            .duration_since(web_time::SystemTime::UNIX_EPOCH)
            .unwrap();
        SystemTime::UNIX_EPOCH + duration
    }

    #[allow(
        clippy::disallowed_methods,
        reason = "`web-time` aliases `std::time::SystemTime::now` on non-WASM platforms"
    )]
    to_std_systemtime(web_time::SystemTime::now())
}

#[cfg(not(feature = "web"))]
pub(crate) fn now() -> SystemTime {
    #[expect(clippy::disallowed_methods, reason = "the `web` feature is disabled")]
    SystemTime::now()
}
