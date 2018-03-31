### v0.8.0

#### Added

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

#### Removed

* `get_ehlo` and `reset` in SmtpTransport are now private

### v0.7.0 (2017-10-08)

#### Added

* Allow validating server certificate

#### Changed

* Split into the *lettre* and *lettre_email* crates
* A lot of small improvements
* Use *tls-native* instead of *openssl* in smtp transport

### v0.6.2 (2017-02-18)

#### Changed

* Update env-logger crate to 0.4
* Update openssl crate to 0.9

### v0.6.1 (2016-10-19)

#### Changed

* #91: Build seperate docs for each release
* #96: Add complete documentation information to README

#### Fixed

* #93: Force building tests before coverage computing
