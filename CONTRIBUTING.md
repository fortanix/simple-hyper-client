# Contribution guidelines

First off, thank you for considering contributing to simple-hyper-client.

If your contribution is not straightforward, please first discuss the change you
wish to make by creating a new issue before making the change.

## Reporting issues

Before reporting an issue on the [issue
tracker](https://github.com/fortanix/simple-hyper-client/issues), please check
that it has not already been reported by searching for some related keywords.

## Pull requests

Try to do one pull request per change.

Please create your commits following the [Conventional Commits
rules](https://www.conventionalcommits.org/en/v1.0.0/#summary). This will enable
you to use some tools to update changelog easily.

### Updating the changelog

Update the changes you have made in
[CHANGELOG](https://github.com/fortanix/simple-hyper-client/blob/main/CHANGELOG.md)
file under the **Unreleased** section.

Add the changes of your pull request to one of the following subsections,
depending on the types of changes defined by [Keep a
changelog](https://keepachangelog.com/en/1.0.0/):

- `Added` for new features.
- `Changed` for changes in existing functionality.
- `Deprecated` for soon-to-be removed features.
- `Removed` for now removed features.
- `Fixed` for any bug fixes.
- `Security` in case of vulnerabilities.

If the required subsection does not exist yet under **Unreleased**, create it!

#### Updating the changelog using tools

If new commits are added following [Conventional Commits
rules](https://www.conventionalcommits.org/en/v1.0.0/#summary), you could use
`cargo changelog` command provided from
[`cargo-smart-release`](https://crates.io/crates/cargo-smart-release) to update
changelogs:

```shell
cargo changelog simple-hyper-client simple-hyper-client-native-tls simple-hyper-client-rustls --write
```

## Developing

### Set up

This is no different than other Rust projects.

```shell
git clone https://github.com/fortanix/simple-hyper-client
cd simple-hyper-client
cargo test
```

### Useful Commands

- Build release version:

  ```shell
  cargo build --release
  ```

- Run Clippy:

  ```shell
  cargo clippy --all-targets --all-features --workspace
  ```

- Run all tests:

  ```shell
  cargo test --all-features --workspace
  ```

- Check to see if there are code formatting issues

  ```shell
  cargo fmt --all -- --check
  ```

- Format the code in the project

  ```shell
  cargo fmt --all
  ```
