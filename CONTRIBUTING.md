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
    refactor: A code change that neither fixes a bug or adds a feature
    perf: A code change that improves performance
    test: Adding missing tests
    chore: Changes to the build process or auxiliary tools and libraries such as documentation generation

**scope** is the lettre part that is being touched. Examples:

    email
    transport-smtp
    transport-file
    transport
    all

The body explains the change, and the footer contains relevant changelog notes and references to fixed issues.
