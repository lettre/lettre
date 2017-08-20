+++
date = "2017-05-21T23:46:17+02:00"
title = "Introduction"
toc = true
weight = 1

+++

{{% notice note %}}
This documentation is written for lettre 0.7, wich has not been released yet.
Please use https://docs.rs/lettre/0.6.2/lettre/ for lettre 0.6.
{{% /notice%}}

Lettre is an email library that allows creating and sending messages. It provides:

* An easy to use email builder
* Pluggable email transports
* Unicode support (for emails and transports, including for sender et recipient addresses when compatible)
* Secure defaults (emails are only sent encrypted by default)
