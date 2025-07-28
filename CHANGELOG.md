<a name="v0.11.18"></a>
### v0.11.18 (2025-07-28)

#### Features

* Allow inline attachments to be named ([#1101])

#### Misc

* Upgrade `socket2` to v0.6 ([#1098])

[#1098]: https://github.com/lettre/lettre/pull/1098
[#1101]: https://github.com/lettre/lettre/pull/1101

<a name="v0.11.17"></a>
### v0.11.17 (2025-06-06)

#### Features

* Add support for `rustls-platform-verifier` ([#1081])

#### Misc

* Change readme example to use `Mailbox::new` instead of string parsing ([#1090])
* Replace futures-util `Mutex` with std `Mutex` in `AsyncStubTransport` ([#1091])
* Avoid duplicate `abort_concurrent` implementation ([#1092])

[#1081]: https://github.com/lettre/lettre/pull/1081
[#1090]: https://github.com/lettre/lettre/pull/1090
[#1091]: https://github.com/lettre/lettre/pull/1091
[#1092]: https://github.com/lettre/lettre/pull/1092

<a name="v0.11.16"></a>
### v0.11.16 (2025-05-12)

#### Features

* Always implement `Clone` for `AsyncFileTransport` ([#1075])

#### Changes

* `Tls`, `CertificateStore`, `TlsParameters`, `TlsParametersBuilder`, `Certificate` and `Identity`
  are now marked as deprecated when no TLS backend is enabled. They will be properly feature gated
  in lettre v0.12 ([#1084])

#### Misc

* Gate `web-time` behind `cfg(target_arch = "wasm32")]` ([#1086])
* Add missing `#[doc(cfg(...))]` attributes ([#1086])
* Upgrade `webpki-roots` to v1 ([#1088])
* Cleanup internal `TlsParameters` and `(Async)NetworkStream` structures ([#1082])
* Feature gate internal `TransportBuilder::tls` to avoid recursive call site warnings ([#1083])
* Fix workaround for embedding cargo script in rustdoc output ([#1077])
* Fix `clippy::io_other_error` warnings ([#1078])
* Upgrade semver compatible dependencies ([#1076], [#1079], [#1080])

[#1075]: https://github.com/lettre/lettre/pull/1075
[#1076]: https://github.com/lettre/lettre/pull/1076
[#1077]: https://github.com/lettre/lettre/pull/1077
[#1078]: https://github.com/lettre/lettre/pull/1078
[#1079]: https://github.com/lettre/lettre/pull/1079
[#1080]: https://github.com/lettre/lettre/pull/1080
[#1082]: https://github.com/lettre/lettre/pull/1082
[#1083]: https://github.com/lettre/lettre/pull/1083
[#1084]: https://github.com/lettre/lettre/pull/1084
[#1086]: https://github.com/lettre/lettre/pull/1086
[#1088]: https://github.com/lettre/lettre/pull/1088

<a name="v0.11.15"></a>
### v0.11.15 (2025-03-10)

#### Upgrade notes

* MSRV is now 1.74 ([#1060])

#### Features

* Add controlled shutdown methods ([#1045], [#1068])

#### Misc

* Deny `unreachable_pub` lint ([#1058])
* Bump minimum supported `rustls` ([#1063])
* Bump minimum supported `serde` ([#1064])
* Upgrade semver compatible dependencies ([#1067])
* Upgrade `email-encoding` to v0.4 ([#1069])

[#1045]: https://github.com/lettre/lettre/pull/1045
[#1058]: https://github.com/lettre/lettre/pull/1058
[#1060]: https://github.com/lettre/lettre/pull/1060
[#1063]: https://github.com/lettre/lettre/pull/1063
[#1064]: https://github.com/lettre/lettre/pull/1064
[#1067]: https://github.com/lettre/lettre/pull/1067
[#1068]: https://github.com/lettre/lettre/pull/1068
[#1069]: https://github.com/lettre/lettre/pull/1069

<a name="v0.11.14"></a>
### v0.11.14 (2025-02-23)

This release deprecates the `rustls-tls`, `tokio1-rustls-tls` and `async-std1-rustls-tls`
features, which will be removed in lettre v0.12.

rustls users should start migrating to the `rustls`, `tokio1-rustls` and
`async-std1-rustls` features. Unlike the deprecated _*rustls-tls_ features,
which automatically enabled the `ring` and `webpki-roots` backends, the new
features do not. To complete the migration, users must either enable the
`aws-lc-rs` or the `ring` feature. Additionally, those who rely on `webpki-roots`
for TLS certificate verification must now explicitly enable its feature.
Users of `rustls-native-certs` do not need to enable `webpki-roots`.

Find out more about the new features via the [lettre rustls docs].

#### Features

* Make it possible to use different `rustls` crypto providers and TLS verifiers ([#1054])

#### Bug fixes

* Use the same `rustls` crypto provider everywhere ([#1055])

#### Misc

* Deprecate `AsyncNetworkStream` being public ([#1059])
* Upgrade `nom` to v8 ([#1048])
* Drop `rustls-pemfile` in favor of `rustls-pki-types` APIs ([#1050])
* Ban direct use of `std::time::SystemTime::now` via clippy ([#1043])
* Drop direct dependency on `rustls-pki-types` ([#1051])
* Remove artifact from `web-time` refactor ([#1049])
* Fix warnings with `rustls-native-certs` when `tracing` is disabled ([#1053])
* Bump license year ([#1057])
* Cleanup `Cargo.toml` style ([#1047])

[lettre rustls docs]: https://docs.rs/lettre/0.11.14/lettre/index.html#smtp-over-tls-via-the-rustls-crate
[#1043]: https://github.com/lettre/lettre/pull/1043
[#1047]: https://github.com/lettre/lettre/pull/1047
[#1048]: https://github.com/lettre/lettre/pull/1048
[#1049]: https://github.com/lettre/lettre/pull/1049
[#1050]: https://github.com/lettre/lettre/pull/1050
[#1051]: https://github.com/lettre/lettre/pull/1051
[#1053]: https://github.com/lettre/lettre/pull/1053
[#1054]: https://github.com/lettre/lettre/pull/1054
[#1055]: https://github.com/lettre/lettre/pull/1055
[#1057]: https://github.com/lettre/lettre/pull/1057
[#1059]: https://github.com/lettre/lettre/pull/1059

<a name="v0.11.13"></a>
### v0.11.13 (2025-02-17)

#### Features

* Add WASM support ([#1037], [#1042])
* Add method to get the TLS verify result with BoringSSL ([#1039])

#### Bug fixes

* Synchronous pool shutdowns being arbitrarily delayed ([#1041])

[#1037]: https://github.com/lettre/lettre/pull/1037
[#1039]: https://github.com/lettre/lettre/pull/1039
[#1041]: https://github.com/lettre/lettre/pull/1041
[#1042]: https://github.com/lettre/lettre/pull/1042

<a name="v0.11.12"></a>
### v0.11.12 (2025-02-02)

#### Misc

* Warn against manually configuring `port` and `tls` on SMTP transport builder ([#1014])
* Document variants of `Tls` enum ([#1015])
* Fix rustdoc warnings ([#1016])
* Add `ContentType::TEXT_PLAIN` to `Message` builder examples ([#1017])
* Document `SmtpTransport` and `AsyncSmtpTransport` ([#1018])
* Fix typo in transport builder `credentials` method ([#1019])
* Document required system dependencies for OpenSSL ([#1030])
* Improve docs for the `transport::smtp` module ([#1031])
* Improve docs for smtp transport builder `from_url` ([#1032])
* Replace `assert!` with `?` on `send` examples ([#1033])
* Warn on more pedantic clippy lints and fix them ([#1035], [#1036])

[#1014]: https://github.com/lettre/lettre/pull/1014
[#1015]: https://github.com/lettre/lettre/pull/1015
[#1016]: https://github.com/lettre/lettre/pull/1016
[#1017]: https://github.com/lettre/lettre/pull/1017
[#1018]: https://github.com/lettre/lettre/pull/1018
[#1019]: https://github.com/lettre/lettre/pull/1019
[#1030]: https://github.com/lettre/lettre/pull/1030
[#1031]: https://github.com/lettre/lettre/pull/1031
[#1032]: https://github.com/lettre/lettre/pull/1032
[#1033]: https://github.com/lettre/lettre/pull/1033
[#1035]: https://github.com/lettre/lettre/pull/1035
[#1036]: https://github.com/lettre/lettre/pull/1036

<a name="v0.11.11"></a>
### v0.11.11 (2024-12-05)

#### Upgrade notes

* MSRV is now 1.71 ([#1008])

#### Bug fixes

* Fix off-by-one error reaching the minimum number of configured pooled connections ([#1012])

#### Misc

* Fix clippy warnings ([#1009])
* Fix `-Zminimal-versions` build ([#1007])

[#1007]: https://github.com/lettre/lettre/pull/1007
[#1008]: https://github.com/lettre/lettre/pull/1008
[#1009]: https://github.com/lettre/lettre/pull/1009
[#1012]: https://github.com/lettre/lettre/pull/1012

<a name="v0.11.10"></a>
### v0.11.10 (2024-10-23)

#### Bug fixes

* Ignore disconnect errors when `pool` feature of SMTP transport is disabled ([#999])
* Use case insensitive comparisons for matching login challenge requests ([#1000])

[#999]: https://github.com/lettre/lettre/pull/999
[#1000]: https://github.com/lettre/lettre/pull/1000

<a name="v0.11.9"></a>
### v0.11.9 (2024-09-13)

#### Bug fixes

* Fix feature gate for `accept_invalid_hostnames` for rustls ([#988])
* Fix parsing `Mailbox` with trailing spaces ([#986])

#### Misc

* Bump `rustls-native-certs` to v0.8 ([#992])
* Make getting started example in readme complete ([#990])

[#988]: https://github.com/lettre/lettre/pull/988
[#986]: https://github.com/lettre/lettre/pull/986
[#990]: https://github.com/lettre/lettre/pull/990
[#992]: https://github.com/lettre/lettre/pull/992

<a name="v0.11.8"></a>
### v0.11.8 (2024-09-03)

#### Features

* Add mTLS support ([#974])
* Implement `accept_invalid_hostnames` for rustls ([#977])
* Provide certificate chain for peer certificates when using `rustls` or `boring-tls` ([#976])

#### Changes

* Make `HeaderName` comparisons via `PartialEq` case insensitive ([#980])

#### Misc

* Fix clippy warnings ([#979])
* Replace manual impl of `#[non_exhaustive]` for `InvalidHeaderName` ([#981])

[#974]: https://github.com/lettre/lettre/pull/974
[#976]: https://github.com/lettre/lettre/pull/976
[#977]: https://github.com/lettre/lettre/pull/977
[#980]: https://github.com/lettre/lettre/pull/980
[#981]: https://github.com/lettre/lettre/pull/981

<a name="v0.11.7"></a>
### v0.11.7 (2024-04-23)

#### Misc

* Bump `hostname` to v0.4 ([#956])
* Fix `tracing` message consistency ([#960])
* Bump minimum required `rustls` to v0.23.5 ([#958])
* Dropped use of `ref` syntax in the entire project ([#959])

[#956]: https://github.com/lettre/lettre/pull/956
[#958]: https://github.com/lettre/lettre/pull/958
[#959]: https://github.com/lettre/lettre/pull/959
[#960]: https://github.com/lettre/lettre/pull/960

<a name="v0.11.6"></a>
### v0.11.6 (2024-03-28)

#### Bug fixes

* Upgraded `email-encoding` to v0.3 - fixing multiple encoding bugs in the process ([#952])

#### Misc

* Updated copyright year in license ([#954])

[#952]: https://github.com/lettre/lettre/pull/952
[#954]: https://github.com/lettre/lettre/pull/954

<a name="v0.11.5"></a>
### v0.11.5 (2024-03-25)

#### Features

* Support SMTP SASL draft login challenge ([#911])
* Add conversion from SMTP response code to integer ([#941])

#### Misc

* Upgrade `rustls` to v0.23 ([#950])
* Bump `base64` to v0.22 ([#945])
* Fix typos in documentation ([#943], [#944])
* Add `Cargo.lock` ([#942])

[#911]: https://github.com/lettre/lettre/pull/911
[#941]: https://github.com/lettre/lettre/pull/941
[#942]: https://github.com/lettre/lettre/pull/942
[#943]: https://github.com/lettre/lettre/pull/943
[#944]: https://github.com/lettre/lettre/pull/944
[#945]: https://github.com/lettre/lettre/pull/945
[#950]: https://github.com/lettre/lettre/pull/950

<a name="v0.11.4"></a>
### v0.11.4 (2024-01-28)

#### Bug fixes

* Percent decode credentials in SMTP connect URL ([#932], [#934])
* Fix mimebody DKIM body-hash computation ([#923])

[#923]: https://github.com/lettre/lettre/pull/923
[#932]: https://github.com/lettre/lettre/pull/932
[#934]: https://github.com/lettre/lettre/pull/934

<a name="v0.11.3"></a>
### v0.11.3 (2024-01-02)

#### Features

* Derive `Clone` for `FileTransport` and `AsyncFileTransport` ([#924])
* Derive `Debug` for `SmtpTransport` ([#925])

#### Misc

* Upgrade `rustls` to v0.22 ([#921])
* Drop once_cell dependency in favor of OnceLock from std ([#928])

[#921]: https://github.com/lettre/lettre/pull/921
[#924]: https://github.com/lettre/lettre/pull/924
[#925]: https://github.com/lettre/lettre/pull/925
[#928]: https://github.com/lettre/lettre/pull/928

<a name="v0.11.2"></a>
### v0.11.2 (2023-11-23)

#### Upgrade notes

* MSRV is now 1.70 ([#916])

#### Misc

* Bump `idna` to v0.5 ([#918])
* Bump `boring` and `tokio-boring` to v4 ([#915])

[#915]: https://github.com/lettre/lettre/pull/915
[#916]: https://github.com/lettre/lettre/pull/916
[#918]: https://github.com/lettre/lettre/pull/918

<a name="v0.11.1"></a>
### v0.11.1 (2023-10-24)

#### Bug fixes

* Fix `webpki-roots` certificate store setup ([#909])

[#909]: https://github.com/lettre/lettre/pull/909

<a name="v0.11.0"></a>
### v0.11.0 (2023-10-15)

While this release technically contains breaking changes, we expect most projects
to be able to upgrade by only bumping the version in `Cargo.toml`.

#### Upgrade notes

* MSRV is now 1.65 ([#869] and [#881])
* `AddressError` is now marked as `#[non_exhaustive]` ([#839])

#### Features

* Improve mailbox parsing ([#839])
* Add construction of SMTP transport from URL ([#901])
* Add `From<Address>` implementation for `Mailbox` ([#879])

#### Misc

* Bump `socket2` to v0.5 ([#868])
* Bump `idna` to v0.4, `fastrand` to v2, `quoted_printable` to v0.5, `rsa` to v0.9 ([#882])
* Bump `webpki-roots` to v0.25 ([#884] and [#890])
* Bump `ed25519-dalek` to v2 fixing RUSTSEC-2022-0093 ([#896])
* Bump `boring`ssl crates to v3 ([#897])

[#839]: https://github.com/lettre/lettre/pull/839
[#868]: https://github.com/lettre/lettre/pull/868
[#869]: https://github.com/lettre/lettre/pull/869
[#879]: https://github.com/lettre/lettre/pull/879
[#881]: https://github.com/lettre/lettre/pull/881
[#882]: https://github.com/lettre/lettre/pull/882
[#884]: https://github.com/lettre/lettre/pull/884
[#890]: https://github.com/lettre/lettre/pull/890
[#896]: https://github.com/lettre/lettre/pull/896
[#897]: https://github.com/lettre/lettre/pull/897
[#901]: https://github.com/lettre/lettre/pull/901

<a name="v0.10.4"></a>
### v0.10.4 (2023-04-02)

#### Misc

* Bumped rustls to 0.21 and all related dependencies ([#867])

[#867]: https://github.com/lettre/lettre/pull/867

<a name="v0.10.3"></a>
### v0.10.3 (2023-02-20)

#### Announcements

It was found that what had been used until now as a basic lettre 0.10
`MessageBuilder::body` example failed to mention that for maximum
compatibility with various email clients a `Content-Type` header
should always be present in the message.

##### Before

```rust
Message::builder()
  // [...] some headers skipped for brevity
  .body(String::from("A plaintext or html body"))?
```

##### Patch

```diff
 Message::builder()
   // [...] some headers skipped for brevity
+  .header(ContentType::TEXT_PLAIN) // or `TEXT_HTML` if the body is html
   .body(String::from("A plaintext or html body"))?
```

#### Features

* Add support for rustls-native-certs when using rustls ([#843])

[#843]: https://github.com/lettre/lettre/pull/843

<a name="v0.10.2"></a>
### v0.10.2 (2023-01-29)

#### Upgrade notes

* MSRV is now 1.60 ([#828])

#### Features

* Allow providing a custom `tokio` stream for `AsyncSmtpTransport` ([#805])
* Return whole SMTP error message ([#821])

#### Bug fixes

* Mailbox displays wrongly when containing a comma and a non-ascii char in its name ([#827])
* Require `quoted_printable` ^0.4.6 in order to fix encoding of tabs and spaces at the end of line ([#837])

#### Misc

* Increase tracing ([#848])
* Bump `idna` to 0.3 ([#816])
* Update `base64` to 0.21 ([#840] and [#851])
* Update `rsa` to 0.8 ([#829] and [#852])

[#805]: https://github.com/lettre/lettre/pull/805
[#816]: https://github.com/lettre/lettre/pull/816
[#821]: https://github.com/lettre/lettre/pull/821
[#827]: https://github.com/lettre/lettre/pull/827
[#828]: https://github.com/lettre/lettre/pull/828
[#829]: https://github.com/lettre/lettre/pull/829
[#837]: https://github.com/lettre/lettre/pull/837
[#840]: https://github.com/lettre/lettre/pull/840
[#848]: https://github.com/lettre/lettre/pull/848
[#851]: https://github.com/lettre/lettre/pull/851
[#852]: https://github.com/lettre/lettre/pull/852

<a name="v0.10.1"></a>
### v0.10.1 (2022-07-20)

#### Features

* Add `boring-tls` support for `SmtpTransport` and `AsyncSmtpTransport`. The latter is only supported with the tokio runtime. ([#797]) ([#798])
* Make the minimum TLS version configurable. ([#799]) ([#800])

#### Bug Fixes

* Ensure connections are closed on abort. ([#801])
* Fix SMTP dot stuffing. ([#803])

[#797]: https://github.com/lettre/lettre/pull/797
[#798]: https://github.com/lettre/lettre/pull/798
[#799]: https://github.com/lettre/lettre/pull/799
[#800]: https://github.com/lettre/lettre/pull/800
[#801]: https://github.com/lettre/lettre/pull/801
[#803]: https://github.com/lettre/lettre/pull/803

<a name="v0.10.0"></a>
### v0.10.0 (2022-06-29)

#### Upgrade notes

Several breaking changes were made between 0.9 and 0.10, but changes should be straightforward:

* MSRV is now 1.56.0
* The `lettre_email` crate has been merged into `lettre`. To migrate, replace `lettre_email` with `lettre::message`
  and make sure to enable the `builder` feature (it's enabled by default).
* `SendableEmail` has been renamed to `Email` and `EmailBuilder::build()` produces it directly. To migrate,
  rename `SendableEmail` to `Email`.
* The `serde-impls` feature has been renamed to `serde`. To migrate, rename the feature.

#### Features

* Add `tokio` 1 support
* Add `rustls` support
* Add `async-std` support. NOTE: native-tls isn't supported when using async-std for the smtp transport.
* Allow enabling multiple SMTP authentication mechanisms
* Allow providing a custom message id
* Allow sending raw emails

#### Breaking Changes

* Merge `lettre_email` into `lettre`
* Merge `Email` and `SendableEmail` into `lettre::message::Email`
* SmtpTransport is now an high level SMTP client. It provides connection pooling and shortcuts for building clients using commonly desired values
* Refactor `TlsParameters` implementation to not expose the internal TLS library
* `FileTransport` writes emails into `.eml` instead of `.json`
* When the hostname feature is disabled or hostname cannot be fetched, `127.0.0.1` is used instead of `localhost` as EHLO parameter (for better RFC compliance and mail server compatibility)
* The `sendmail` and `file` transports aren't enabled by default anymore.
* The `new` method of `ClientId` is deprecated
* Rename `serde-impls` feature to `serde`
* The `SendmailTransport` now uses the `sendmail` command in current `PATH` by default instead of
  `/usr/bin/sendmail`.

#### Bug Fixes

* Fix argument injection in `SendmailTransport` (see [RUSTSEC-2020-0069](https://github.com/RustSec/advisory-db/blob/master/crates/lettre/RUSTSEC-2020-0069.md))
* Correctly encode header values containing non-ASCII characters
* Timeout bug causing infinite hang
* Fix doc tests in website
* Fix docs for `domain` field

#### Misc

* Improve documentation, examples and tests
* Replace `line-wrap`, `email`, `bufstream` with our own implementations
* Remove `bytes`
* Remove `time`
* Remove `fast_chemail`
* Update `base64` to 0.13
* Update `hostname` to 0.3
* Update to `nom` 6
* Replace `log` with `tracing`
* Move CI to GitHub Actions
* Use criterion for benchmarks

<a name="v0.9.2"></a>
### v0.9.2 (2019-06-11)

#### Bug Fixes

* **email:**
  * Fix compilation with Rust 1.36+ ([393ef8d](https://github.com/lettre/lettre/commit/393ef8dcd1b1c6a6119d0666d5f09b12f50f6b4b))

<a name="v0.9.1"></a>
### v0.9.1 (2019-05-05)

#### Features

* **email:**
  * Re-export mime crate ([a0c8fb9](https://github.com/lettre/lettre/commit/a0c8fb9))

<a name="v0.9.0"></a>
### v0.9.0 (2019-03-17)

#### Bug Fixes

* **email:**
  * Inserting 'from' from envelope into message headers ([058fa69](https://github.com/lettre/lettre/commit/058fa69))
  * Do not include Bcc addresses in headers ([ee31bbe](https://github.com/lettre/lettre/commit/ee31bbe))

* **transport:**
  * Write timeout is not set in smtp transport ([d71b560](https://github.com/lettre/lettre/commit/d71b560))
  * Client::read_response infinite loop ([72f3cd8](https://github.com/lettre/lettre/commit/72f3cd8))

#### Features

* **all:**
  * Update dependencies
  * Start using the failure crate for errors ([c10fe3d](https://github.com/lettre/lettre/commit/c10fe3d))

* **transport:**
  * Remove TLS 1.1 in accepted protocols by default (only allow TLS 1.2) ([4b48bdb](https://github.com/lettre/lettre/commit/4b48bdb))
  * Initial support for XOAUTH2 ([ed7c164](https://github.com/lettre/lettre/commit/ed7c164))
  * Remove support for CRAM-MD5 ([bc09aa2](https://github.com/lettre/lettre/commit/bc09aa2))
  * SMTP connection pool implementation with r2d2 ([434654e](https://github.com/lettre/lettre/commit/434654e))
  * Use md-5 and hmac instead of rust-crypto ([e7e0f34](https://github.com/lettre/lettre/commit/e7e0f34))
  * Gmail transport simple example ([a8d8e2a](https://github.com/lettre/lettre/commit/a8d8e2a))

* **email:**
  * Add In-Reply-To and References headers ([fc91bb6](https://github.com/lettre/lettre/commit/fc91bb6))
  * Remove non-chaining builder methods ([1baf8a9](https://github.com/lettre/lettre/commit/1baf8a9))

<a name="v0.8.2"></a>
### v0.8.2 (2018-05-03)


#### Bug Fixes

* **transport:**  Write timeout is not set in smtp transport ([cc3580a8](https://github.com/lettre/lettre/commit/cc3580a8942e11c2addf6677f05e16fb451c7ea0))

#### Style

* **all:**  Fix typos ([360c42ff](https://github.com/lettre/lettre/commit/360c42ffb8f706222eaad14e72619df1e4857814))

#### Features

* **all:**
  *  Add set -xe option to build scripts ([57bbabaa](https://github.com/lettre/lettre/commit/57bbabaa6a10cc1a4de6f379e25babfee7adf6ad))
  *  Move post-success scripts to separate files ([3177b58c](https://github.com/lettre/lettre/commit/3177b58c6d11ffae73c958713f6f0084173924e1))
  *  Add website upload to travis build script ([a5294df6](https://github.com/lettre/lettre/commit/a5294df63728e14e24eeb851bb4403abd6a7bd36))
  *  Add codecov upload in travis ([a03bfa00](https://github.com/lettre/lettre/commit/a03bfa008537b1d86ff789d0823e89ad5d99bd79))
  *  Update README to put useful links at the top ([1ebbe660](https://github.com/lettre/lettre/commit/1ebbe660f5e142712f702c02d5d1e45211763b42))
  *  Update badges in README and Cargo.toml ([f7ee5c42](https://github.com/lettre/lettre/commit/f7ee5c427ad71e4295f2f1d8e3e9e2dd850223e8))
  *  Move docs from hugo to gitbook ([27935e32](https://github.com/lettre/lettre/commit/27935e32ef097db8db004569f35cad1d6cd30eca))
* **transport:**  Use md-5 and hmac instead of rust-crypto ([0cf018a8](https://github.com/lettre/lettre/commit/0cf018a85e4ea1ad16c7216670da560cc915ec32))



<a name="v0.8.1"></a>
### v0.8.1 (2018-04-11)

#### Fix

* **all:**
  *  Replace skeptic by some custom rustdoc invocations ([81bad131](https://github.com/lettre/lettre/commit/81bad1317519d330c46ea02f2b7a266b97cc00dd))

#### Documentation

* **all:**
  *  Add changelog sections for style and docs ([b4d03ead](https://github.com/lettre/lettre/commit/b4d03ead8cce04e0c3d65a30e7a07acca9530f30))
  *  Use clog to generate changelogs ([8981a775](https://github.com/lettre/lettre/commit/8981a7758c89be69974ef204c4390744aea94e4f), closes [#233](https://github.com/lettre/lettre/issues/233))

#### Style

* **transport-smtp:**  Avoid useless empty format strings ([f3271715](https://github.com/lettre/lettre/commit/f3271715ecaf2793c9064462184867e4f22b0ead))



<a name="v0.8.0"></a>
### v0.8.0 (2018-03-31)

#### Added

* Support binary files as attachment
* Move doc to a dedicated website
* Add tests for the doc using skeptic
* Added a code of conduct
* Use hostname as `ClientId` when available

#### Changed

* Detail in SMTP Response is now an enum
* Use nom for parsing smtp responses
* `Envelope` was moved from `lettre_email` to `lettre`
* `EmailAddress::new()` now returns a `Result`
* `SendableEmail` replaces `from` and `to` by `envelope` that returns an `Envelope`
* `File` transport storage format has changed

#### Fixed

* Add missing "Bcc" headers when building the email
* Specify utf-8 charset for html
* Use parts for text and html methods to work with attachments

#### Removed

* `get_ehlo` and `reset` in SmtpTransport are now private

<a name="v0.7.0"></a>
### v0.7.0 (2017-10-08)

#### Added

* Allow validating server certificate
* Initial (incomplete) attachments support

#### Changed

* Split into the *lettre* and *lettre_email* crates
* A lot of small improvements
* Use *tls-native* instead of *openssl* in smtp transport

<a name="v0.6.2"></a>
### v0.6.2 (2017-02-18)

#### Changed

* Update env-logger crate to 0.4
* Update openssl crate to 0.9
* Update uuid crate to 0.4

<a name="v0.6.1"></a>
### v0.6.1 (2016-10-19)

#### Changes

* **documentation**
  * #91: Build separate docs for each release
  * #96: Add complete documentation information to README

#### Fixed

* #85: Use address-list for "To", "From" etc.
* #93: Force building tests before coverage computing

<a name="v0.6.0"></a>
### v0.6.0 (2016-05-05)

#### Changes

*  multipart support
*  add non-consuming methods for Email builders
* `add_header` does not return the builder anymore,
  for consistency with other methods. Use the `header`
  method instead
