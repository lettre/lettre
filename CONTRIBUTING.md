## Contributing to Lettre

The following guidelines are inspired from the [hyper project](https://github.com/hyperium/hyper/blob/master/CONTRIBUTING.md).

### Code formatting

All code must be formatted using `rustfmt`.

### Commit Message Format

Each commit message consists of a header, a body and a footer. The header has a special format that includes a type, a scope and a subject:

```text
<type>(<scope>): <subject> <BLANK LINE> <body> <BLANK LINE> <footer>
```

Any line of the commit message cannot be longer 72 characters.

**type** must be one of the following:

    feat: A new feature
    fix: A bug fix
    docs: Documentation only changes
    style: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
    perf: A code change that improves performance

**scope** is the lettre part that is being touched. Examples:

    email
    transport-smtp
    transport-file
    transport
    all

The body explains the change, and the footer contains relevant changelog notes and references to fixed issues.

### Release process

Releases are made using `cargo-release`:

```bash
cargo release --dry-run 0.10.0 --prev-tag-name v0.9.2 -v
```
