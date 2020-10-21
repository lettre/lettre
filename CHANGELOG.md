### v0.6.3 (2022-1--21)

* **transport**: Allow using openssl 0.10

### v0.6.2 (2017-02-18)

#### Features

* **all**
  * Update uuid crate to 0.4
  * Update env-logger crate to 0.4
  * Update openssl crate to 0.9

### v0.6.1 (2016-10-19)

#### Features

* **documentation**
  * #91: Build seperate docs for each release
  * #96: Add complete documentation information to README

#### Bugfixes

* **email**
  * #85: Use address-list for "To", "From" etc.

* **tests**
  * #93: Force building tests before coverage computing

### v0.6.0 (2016-05-05)

#### Features

* **email**
  *  multipart support
  *  add non-consuming methods for Email builders

#### Beaking Change

* **email**
  * `add_header` does not return the builder anymore, 
    for consistency with other methods. Use the `header`
    method instead
