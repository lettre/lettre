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
* Move CI to Github Actions
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
