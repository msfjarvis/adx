# Running tests

To avoid network requests during testing the project uses a somewhat rudimentary hack around conditional compilation to load up a dump of `maven.google.com` from the repository itself. To ensure that all tests are passing, make sure to run them as following:

```bash
RUSTFLAGS='--cfg nix_check' cargo build
RUSTFLAGS='--cfg nix_check' cargo nextest run
```
