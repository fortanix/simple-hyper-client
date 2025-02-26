# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Documentation

 - <csr-id-96e112b86843ed5732b17a3d909900db40cf3a93/> add changelog
   Add changelogs using `cargo changelog simple-hyper-client simple-hyper-client-native-tls simple-hyper-client-rustls --write`.

### New Features

 - <csr-id-172270d826a1ddc94bb455be1f59f22b54a89544/> add simple-hyper-client-rustls
   - Add  simple-hyper-client-rustls crate for help using simple-hyper-client with tokio-rustls.
 - <csr-id-f6a4928f7fca8b5746de5d463f876f78b74a6dd9/> expose connector types at crate root
   Re-export `HttpOrHttpsConnection` and `HttpsConnector` at the crate root
   for easier access. This makes the types available without needing to
   reference the `connector` module explicitly.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Expose connector types at crate root ([`f6a4928`](https://github.com/fortanix/simple-hyper-client/commit/f6a4928f7fca8b5746de5d463f876f78b74a6dd9))
    - Add changelog ([`96e112b`](https://github.com/fortanix/simple-hyper-client/commit/96e112b86843ed5732b17a3d909900db40cf3a93))
    - Add simple-hyper-client-rustls ([`172270d`](https://github.com/fortanix/simple-hyper-client/commit/172270d826a1ddc94bb455be1f59f22b54a89544))
</details>

