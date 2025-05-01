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

pub trait TlsBackend: private::Sealed {
    type Certificate;
    type Identity;
}

#[cfg(feature = "boring-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
#[non_exhaustive]
pub struct BoringTls;

#[cfg(feature = "boring-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "boring-tls")))]
impl TlsBackend for BoringTls {
    type Certificate = self::boring_tls::Certificate;
    type Identity = self::boring_tls::Identity;
}

#[cfg(feature = "native-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
#[non_exhaustive]
pub struct NativeTls;

#[cfg(feature = "native-tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "native-tls")))]
impl TlsBackend for NativeTls {
    type Certificate = self::native_tls::Certificate;
    type Identity = self::native_tls::Identity;
}

#[cfg(feature = "rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
#[non_exhaustive]
pub struct Rustls;

#[cfg(feature = "rustls")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustls")))]
impl TlsBackend for Rustls {
    type Certificate = self::rustls::Certificate;
    type Identity = self::rustls::Identity;
}

mod private {
    // FIXME: this should be `pub(super)` but the `private_bounds` lint doesn't like it
    pub trait Sealed {}

    #[cfg(feature = "boring-tls")]
    impl Sealed for super::BoringTls {}

    #[cfg(feature = "native-tls")]
    impl Sealed for super::NativeTls {}

    #[cfg(feature = "rustls")]
    impl Sealed for super::Rustls {}
}
