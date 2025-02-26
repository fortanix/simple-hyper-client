# Introduction

This workspace provides crates:

- [`simple-hyper-client`](./simple-hyper-client/): a wrapper for [hyper's
  `Client` type] providing a simpler async interface as well as a blocking
  interface. For a more feature-rich HTTP client use [`reqwest`] instead.
- [`simple-hyper-client-native-tls`](./simple-hyper-client-native-tls/):
  Provides [`NetworkConnector`] implementation using
  [tokio-native-tls](https://crates.io/crates/tokio-native-tls).
- [`simple-hyper-client-rustls`](./simple-hyper-client-rustls/): Provides
  [`NetworkConnector`] implementation using [tokio-rustls].

## Example

Please check crate [`example`](./example/):

- Async clients:
  - Use native-tls: `cargo run -p example --bin native_tls_client_async`
  - Use rustls: `cargo run -p example --bin rustls_client_async`
- Blocking clients:
  - Use native-tls: `cargo run -p example --bin native_tls_client`
  - Use rustls: `cargo run -p example --bin rustls_client`

## Contributing

We gratefully accept bug reports and contributions from the community. By
participating in this community, you agree to abide by [Code of
Conduct](./CODE_OF_CONDUCT.md). All contributions are covered under the
Developer's Certificate of Origin (DCO).

Please check [CONTRIBUTING.md](./CONTRIBUTING.md) for more details.

## Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I have the right
to submit it under the open source license indicated in the file; or

(b) The contribution is based upon previous work that, to the best of my
knowledge, is covered under an appropriate open source license and I have the
right under that license to submit that work with modifications, whether created
in whole or in part by me, under the same open source license (unless I am
permitted to submit under a different license), as indicated in the file; or

(c) The contribution was provided directly to me by some other person who
certified (a), (b) or (c) and I have not modified it.

(d) I understand and agree that this project and the contribution are public and
that a record of the contribution (including all personal information I submit
with it, including my sign-off) is maintained indefinitely and may be
redistributed consistent with this project or the open source license(s)
involved.

## License

This project is primarily distributed under the terms of the Mozilla Public
License (MPL) 2.0, see [LICENSE](./LICENSE) for details.

[hyper's `Client` type]:
    https://docs.rs/hyper/latest/hyper/client/struct.Client.html
[`reqwest`]: https://crates.io/crates/reqwest
[`NetworkConnector`]:
    https://docs.rs/simple-hyper-client/latest/simple_hyper_client/trait.NetworkConnector.html
[tokio-rustls]: https://crates.io/crates/tokio-rustls