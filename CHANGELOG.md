<a name="v0.10.0"></a>
### v0.10.0 (unreleased)

#### Upgrade notes

Several breaking changes were made between 0.9 and 0.10, but changes should be straightforward:

* The `lettre_email` crate has been merged into `lettre`. To migrate, replace `lettre_email` with `lettre::builder`
  and make sure to enable the `builder` feature (it's enabled by default).
* `SendableEmail` has been renamed to `Email` and `EmailBuilder::build()` produces it directly. To migrate,
  rename `SendableEmail` to `Email`.
* The `serde-impls` feature has been renamed to `serde`. To migrate, rename the feature.

#### Features

* Add `rustls` support ([29e4829](https://github.com/lettre/lettre/commit/29e4829), [39a0686](https://github.com/lettre/lettre/commit/39a0686))
* Allow providing a custom message id ([50d96ad](https://github.com/lettre/lettre/commit/50d96ad))
* Add `EmailAddress::is_valid` and `into_inner` ([e5a1248](https://github.com/lettre/lettre/commit/e5a1248))
* Accept `Into<SendableEmail>` ([86e5181](https://github.com/lettre/lettre/commit/86e5181))
* Allow forcing of a specific auth ([bf2adca](https://github.com/lettre/lettre/commit/bf2adca))
* Add `build_body` ([e927d0b](https://github.com/lettre/lettre/commit/e927d0b))

#### Changes

* Move CI to Github Actions ([3eef024](https://github.com/lettre/lettre/commit/3eef024))
* MSRV is now 1.36 ([d227cd4](https://github.com/lettre/lettre/commit/d227cd4))
* Merged `lettre_email` into `lettre` ([0f3f27f](https://github.com/lettre/lettre/commit/0f3f27f))
* Rename `serde-impls` feature to `serde` ([aac3e00](https://github.com/lettre/lettre/commit/aac3e00))
* Use criterion for benchmarks ([eda7fc1](https://github.com/lettre/lettre/commit/eda7fc1))
* Update to nom 5 ([5bc1cba](https://github.com/lettre/lettre/commit/5bc1cba))
* Change website url schemes to https ([6014f5c](https://github.com/lettre/lettre/commit/6014f5c))
* Use serde's `derive` feature instead of the `serde_derive` crate ([4fbe700](https://github.com/lettre/lettre/commit/4fbe700))
* Merge `Email` and `SendableEmail` into `lettre::Email` ([ce37464](https://github.com/lettre/lettre/commit/ce37464))
* When the hostname feature is disabled or hostname cannot be fetched, `127.0.0.1` is used instead of `localhost` as
  EHLO parameter (for better RFC compliance and mail server compatibility)
* The `new` method of `ClientId` is deprecated

#### Bug Fixes

* Timeout bug causing infinite hang ([6eff9d3](https://github.com/lettre/lettre/commit/6eff9d3))
* Fix doc tests in website ([947af0a](https://github.com/lettre/lettre/commit/947af0a))
* Fix docs for `domain` field ([0e05e0e](https://github.com/lettre/lettre/commit/0e05e0e))

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
