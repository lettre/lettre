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
