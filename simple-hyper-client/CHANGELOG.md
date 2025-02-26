# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Documentation

 - <csr-id-96e112b86843ed5732b17a3d909900db40cf3a93/> add changelog
   Add changelogs using `cargo changelog simple-hyper-client simple-hyper-client-native-tls simple-hyper-client-rustls --write`.

### New Features (BREAKING)

 - <csr-id-c1274bfc9f43f77b5a4fe612e78fa6db4d6b4aff/> decouple native-tls connector logic from simple-hyper-client
   - Move native-tls HttpsConnector logic into separate crate: `simple-hyper-client-native-tls`.
- Convert repo into a rust workspace instead of single crate repo.
- Bump minor version of `simple-hyper-client`.
- Remove original CHANGELOG file.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Add changelog ([`96e112b`](https://github.com/fortanix/simple-hyper-client/commit/96e112b86843ed5732b17a3d909900db40cf3a93))
    - Decouple native-tls connector logic from simple-hyper-client ([`c1274bf`](https://github.com/fortanix/simple-hyper-client/commit/c1274bfc9f43f77b5a4fe612e78fa6db4d6b4aff))
</details>

