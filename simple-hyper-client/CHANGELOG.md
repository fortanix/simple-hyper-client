# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

 - <csr-id-d711be9504d42b20f3b3ef261a354fbb326d1201/> Add support for wrapping arbitrary request bodies
 - <csr-id-f7c53d53eced71c250ad703379d17893685d9745/> Upgrade `hyper` to v1

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 29 commits contributed to the release.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #86 from fortanix/fix/improve-error-messages ([`7d8664f`](https://github.com/fortanix/simple-hyper-client/commit/7d8664fc52bf36aceffb5ff1828bd429f2f4a6b3))
    - Use `Display` for the source error ([`9f42129`](https://github.com/fortanix/simple-hyper-client/commit/9f42129db7b996152e411f0648e93b1118d39747))
    - Improve error messages ([`5ba8bb7`](https://github.com/fortanix/simple-hyper-client/commit/5ba8bb7637416102c27af653b6bb717f093cba47))
    - Merge pull request #84 from fortanix/pieagt/reintroduce-shared-body ([`cd423fa`](https://github.com/fortanix/simple-hyper-client/commit/cd423fad84138598861a40c83e99387ba468a1c8))
    - Ensure that wrapped bodies still have an accurate size hint ([`b9a0d12`](https://github.com/fortanix/simple-hyper-client/commit/b9a0d12825447d2131fb5b3d3d68f1763ed501c3))
    - Re-introduce the SharedBody type ([`17d480b`](https://github.com/fortanix/simple-hyper-client/commit/17d480bd25f9afd83a6111235906f4ffb24a8af9))
    - Merge pull request #81 from fortanix/pieagt/add-support-for-wrapping-arbitrary-bodies ([`d711be9`](https://github.com/fortanix/simple-hyper-client/commit/d711be9504d42b20f3b3ef261a354fbb326d1201))
    - Add test cases for GET requests with/without bodies ([`d7fc726`](https://github.com/fortanix/simple-hyper-client/commit/d7fc726333eac18ca804c15bded47ad4f8fa4028))
    - Add convenience methods to Error type ([`410101e`](https://github.com/fortanix/simple-hyper-client/commit/410101eb86c77905427aeb508aa52b89867f0f60))
    - Merge pull request #80 from fortanix/ivan/relax-dependencies ([`4fd6a3f`](https://github.com/fortanix/simple-hyper-client/commit/4fd6a3f2214405de4eb137b7f36e36c50534dd88))
    - Don't error out with BodyNotAllowed on empty request bodies ([`f0c10e6`](https://github.com/fortanix/simple-hyper-client/commit/f0c10e69860043eb1beebe4afd82355ee748ed1a))
    - Test that ContentLength header is set with requests ([`62aecd6`](https://github.com/fortanix/simple-hyper-client/commit/62aecd6e216e5ad3ab1d7fcda592fca7d9b1509f))
    - Add support for wrapping arbitrary bodies as simple hyper client request bodies ([`f1e9cff`](https://github.com/fortanix/simple-hyper-client/commit/f1e9cff1dea765c2a6d897ebc3e43306292ea27e))
    - Relax dependency versions ([`6b5c4c3`](https://github.com/fortanix/simple-hyper-client/commit/6b5c4c3942da06caeb47833cc04fa3582781890f))
    - Merge pull request #78 from fortanix/ivan/upgrade-hyper-to-v1 ([`9cbf6a4`](https://github.com/fortanix/simple-hyper-client/commit/9cbf6a49df1da48445f42cc67794c126125ff066))
    - Address code review comments ([`1fbbaf3`](https://github.com/fortanix/simple-hyper-client/commit/1fbbaf32e2be585c2358fe964f2e2c725022e1c6))
    - Upgrade `hyper` to v1 ([`f7c53d5`](https://github.com/fortanix/simple-hyper-client/commit/f7c53d53eced71c250ad703379d17893685d9745))
    - Merge pull request #60 from fortanix/dependabot/cargo/tokio-1.50.0 ([`1de4130`](https://github.com/fortanix/simple-hyper-client/commit/1de4130ca0b704f641daeb63ba67b7d686a8fdfe))
    - Bump tokio from 1.49.0 to 1.50.0 ([`a03613b`](https://github.com/fortanix/simple-hyper-client/commit/a03613bf5ec4eea2433b095778b1f5d0ff72f896))
    - Merge pull request #58 from fortanix/dependabot/cargo/futures-executor-0.3.32 ([`c03a805`](https://github.com/fortanix/simple-hyper-client/commit/c03a805a4068159cc3e2a8be2e358d0c3be17fa5))
    - Merge pull request #59 from fortanix/dependabot/cargo/futures-util-0.3.32 ([`162d828`](https://github.com/fortanix/simple-hyper-client/commit/162d828bb9371e54cca5d5689a634ab8e82adb18))
    - Bump futures-util from 0.3.31 to 0.3.32 ([`125a4f6`](https://github.com/fortanix/simple-hyper-client/commit/125a4f6cfaa0a1e8a47295df5d63777706485177))
    - Bump futures-executor from 0.3.31 to 0.3.32 ([`282df91`](https://github.com/fortanix/simple-hyper-client/commit/282df9140c5c6a5ca06c1b5e14bcb8a110b1e3d8))
    - Merge pull request #51 from fortanix/dependabot/cargo/tokio-1.49.0 ([`3da5b84`](https://github.com/fortanix/simple-hyper-client/commit/3da5b84ff1f6ef3e8f1dcb185b3977e383ca83ee))
    - Bump tokio from 1.45.1 to 1.49.0 ([`9965b91`](https://github.com/fortanix/simple-hyper-client/commit/9965b91b8f18f632a0080945a07347c733128e7f))
    - Merge pull request #52 from fortanix/dependabot/cargo/tokio-stream-0.1.18 ([`b4bf755`](https://github.com/fortanix/simple-hyper-client/commit/b4bf7553e9579f1ee04cc841560913780f8b5dfa))
    - Bump tokio-stream from 0.1.17 to 0.1.18 ([`981b9e7`](https://github.com/fortanix/simple-hyper-client/commit/981b9e780f521ee54b24e2a26ceb2f0e7ae1ed51))
    - Merge pull request #34 from fortanix/dependabot/cargo/tokio-1.45.1 ([`4bc2d94`](https://github.com/fortanix/simple-hyper-client/commit/4bc2d944f8d16fb6e3ed65d3c46229c9d9c2ed91))
    - Bump tokio from 1.43.0 to 1.45.1 ([`e3ec8b8`](https://github.com/fortanix/simple-hyper-client/commit/e3ec8b8a0d7b2b56fb0f9a1534b55de56e67b0a3))
</details>

## 0.2.0 (2025-02-28)

### New Features (BREAKING)

 - <csr-id-c1274bfc9f43f77b5a4fe612e78fa6db4d6b4aff/> decouple native-tls connector logic from simple-hyper-client
   - Move native-tls HttpsConnector logic into separate crate: `simple-hyper-client-native-tls`.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 451 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #12 from fortanix/yx/add-rustls-support_part2 ([`e244718`](https://github.com/fortanix/simple-hyper-client/commit/e244718eb23e01dce325c9ffdb5cb54d6a8cd824))
    - Update changelog ([`9ffac4f`](https://github.com/fortanix/simple-hyper-client/commit/9ffac4f132e370397618268fe1adbaf774eaa5fd))
    - Feat: decouple native-tls connector logic from simple-hyper-client ([`eae276d`](https://github.com/fortanix/simple-hyper-client/commit/eae276d75b2cb7831a40651fcd85db34a135f326))
</details>

## 0.1.3 (2023-12-04)

## 0.1.2 (2023-10-17)

## 0.1.1 (2023-03-28)

## 0.1.0 (2022-03-14)

