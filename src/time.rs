use std::time::SystemTime;

#[cfg(feature = "web")]
pub(crate) fn now() -> SystemTime {
    fn to_std_systemtime(time: web_time::SystemTime) -> std::time::SystemTime {
        let duration = time
            .duration_since(web_time::SystemTime::UNIX_EPOCH)
            .unwrap();
        SystemTime::UNIX_EPOCH + duration
    }

    to_std_systemtime(web_time::SystemTime::now())
}

#[cfg(not(feature = "web"))]
pub(crate) fn now() -> SystemTime {
    SystemTime::now()
}
