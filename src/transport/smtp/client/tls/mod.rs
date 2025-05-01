#[cfg(feature = "boring-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
pub mod boring_tls;
pub(super) mod deprecated;
#[cfg(feature = "native-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
pub mod native_tls;
#[cfg(feature = "rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
pub mod rustls;
