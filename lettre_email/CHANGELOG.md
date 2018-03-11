### v0.8.0

#### Added

* Support binary files as attachment
* Move doc to a dedicated website
* Add tests for the doc using skeptic
* Added a code of conduct

#### Fixed

* Add missing "Bcc" headers when building the email
* Specify utf-8 charset for html
* Use parts for text and html methods to work with attachments

### v0.7.0 (2017-10-08)

#### Added

* Initial (incomplete) attachments support

#### Changes

* Split into the *lettre* and *lettre_email* crates

### v0.6.2 (2017-02-18)

#### Features

#### Changes

* Update uuid crate to 0.4

### v0.6.1 (2016-10-19)

#### Changes

* **documentation**
  * #91: Build seperate docs for each release
  * #96: Add complete documentation information to README

#### Fixed

* #85: Use address-list for "To", "From" etc.
* #93: Force building tests before coverage computing

### v0.6.0 (2016-05-05)

#### Changes

*  multipart support
*  add non-consuming methods for Email builders
* `add_header` does not return the builder anymore, 
  for consistency with other methods. Use the `header`
  method instead
