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
