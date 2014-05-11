Rust SMTP library
=================

.. image:: https://travis-ci.org/amousset/rust-smtp.png?branch=master
   :target: https://travis-ci.org/amousset/rust-smtp

This library implements an SMTP client, and maybe later a simple SMTP server.

Rust versions
-------------

This library is designed for Rust 0.11-pre (master).

Install
------

Build the library:

    make

To build the example command-line client code:

    make examples

To run the example's help:

    ./build/client -h

Todo
----

- RFC compliance
- SSL/TLS support
- AUTH support

License
-------

This program is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See LICENSE-APACHE, LICENSE-MIT, and COPYRIGHT for details.
