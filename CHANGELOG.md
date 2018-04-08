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
  * #91: Build seperate docs for each release
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
