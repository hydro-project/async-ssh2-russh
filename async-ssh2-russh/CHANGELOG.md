

## v0.4.0 (2026-11-19)

### Bug Fixes (BREAKING)

 - <csr-id-e1509365d47ed31dc50e64b89b6d20f8be8d174a/> update russh 0.65, fix RUSTSEC-2026-0153 and RUSTSEC-2026-0154
   `russh` before `0.60.3` is affected by
   [RUSTSEC-2026-0153](https://rustsec.org/advisories/RUSTSEC-2026-0153.html) (`russh-cryptovec`)
   and [RUSTSEC-2026-0154](https://rustsec.org/advisories/RUSTSEC-2026-0154.html) (`russh`),
   CVE-2026-46673 / GHSA-g9f8-wqj9-fjw5, so the `russh 0.55` requirement could not resolve
   to a patched version.
   
   - `russh` `0.55.0` -> `0.65.2`, `russh-sftp` `2.1.0` -> `3.0.1`.
   - MSRV `1.85.0` -> `1.89.0`, required by `russh 0.65`.
   - `russh 0.65` delivers channel data as `bytes::Bytes` instead of `CryptoVec`, and passes
     `PublicKeyOrCertificate` to `Handler::check_server_key`.
   - Fix `ReadStream` panics: `AsyncRead::poll_read` bounded the copy by `ReadBuf::capacity()`
     instead of `remaining()`, so `put_slice` panicked on a partially filled `ReadBuf` (e.g.
     `read_to_end`); `Read::read` copied into all of `buf`, so `copy_from_slice` panicked whenever
     `buf` was longer than the buffered data.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 337 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#5](https://github.com/hydro-project/async-ssh2-russh/issues/5)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#5](https://github.com/hydro-project/async-ssh2-russh/issues/5)**
    - Update russh 0.65, fix RUSTSEC-2026-0153 and RUSTSEC-2026-0154 ([`e150936`](https://github.com/hydro-project/async-ssh2-russh/commit/e1509365d47ed31dc50e64b89b6d20f8be8d174a))
</details>

## v0.3.0 (2025-12-16)

### Chore

 - <csr-id-e04b17c55d598d839d0dc9c4ad7b78a66f56b02e/> update russh 0.55, fix cve
   Fix cve: https://nvd.nist.gov/vuln/detail/CVE-2025-54804

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 5 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#3](https://github.com/hydro-project/async-ssh2-russh/issues/3)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#3](https://github.com/hydro-project/async-ssh2-russh/issues/3)**
    - Update russh 0.55, fix cve ([`e04b17c`](https://github.com/hydro-project/async-ssh2-russh/commit/e04b17c55d598d839d0dc9c4ad7b78a66f56b02e))
 * **Uncategorized**
    - Release async-ssh2-russh v0.3.0 ([`a79f8a0`](https://github.com/hydro-project/async-ssh2-russh/commit/a79f8a0686a3c71e2049381591eb37d20a79976c))
</details>

## v0.2.0 (2025-12-11)

<csr-id-c4dbc623c7988850166c1182f496fa697f8ca544/>
<csr-id-90a233a994960672dd5fe876f82f45e030a2f576/>

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 226 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#2](https://github.com/hydro-project/async-ssh2-russh/issues/2)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#2](https://github.com/hydro-project/async-ssh2-russh/issues/2)**
    - Convert `fn closed` to promise to allow `&self` ([`90a233a`](https://github.com/hydro-project/async-ssh2-russh/commit/90a233a994960672dd5fe876f82f45e030a2f576))
 * **Uncategorized**
    - Release async-promise v0.1.1, async-ssh2-russh v0.2.0 ([`80b2a60`](https://github.com/hydro-project/async-ssh2-russh/commit/80b2a60e61cdc367137407655d428e44b5406594))
    - Bump versions for release ([`c4dbc62`](https://github.com/hydro-project/async-ssh2-russh/commit/c4dbc623c7988850166c1182f496fa697f8ca544))
</details>

## v0.1.0 (2025-04-29)

### New Features

 - <csr-id-5c0437695e7f0b1a4b3bea5af3e138de8a85a12e/> implement `async-ssh2-russh`
   From https://github.com/hydro-project/async-ssh2-russh

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release async-promise v0.1.0, async-ssh2-russh v0.1.0 ([`20e4970`](https://github.com/hydro-project/async-ssh2-russh/commit/20e4970dbbe9db3e101f3066098e44e4f3050f79))
    - Implement `async-ssh2-russh` ([`5c04376`](https://github.com/hydro-project/async-ssh2-russh/commit/5c0437695e7f0b1a4b3bea5af3e138de8a85a12e))
</details>

