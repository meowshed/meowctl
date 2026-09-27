---
id: SPC-1800
artifact: spec
status: live
revised: 2026-09-27
checked-at:
states: [REQ-1801, REQ-1802, REQ-1803, REQ-1804, REQ-1805, REQ-1806, REQ-1810, REQ-1811, REQ-1812, REQ-1813, REQ-1814, REQ-1900, REQ-1901, REQ-1902, REQ-1903, REQ-1904, REQ-1905]
---

# The network

## Scope

This component owns every request meowctl makes, behind one trait. It is the
third effect, beside the filesystem and processes, and it is a trait for the
same two reasons those are: a test must be able to answer a request without a
server, and a caller must not be able to reach the network by some other route
(from docs/spec/net.md, high). It knows nothing about modules, registries or
tarballs: it fetches bytes from a URL and says why it couldn't (from
docs/spec/net.md, high).

It lives in the crate `meowctl-net`, its design is
`docs/design/0.2.0-rust-rewrite.md` §3, and its `v0.1.0` equivalent is the
`*http.Client` fields on `RegistryLoader`, `githubLoader` and `ctx.download`
(from docs/spec/net.md, high).

## Boundary

The `Http` trait, its implementations, and the error a failed request produces
(from docs/spec/net.md, high).

| Surface | What it is |
| --- | --- |
| `Http` | One method: fetch a URL and return its body as bytes (from docs/spec/net.md, high) |
| `RealHttp`, `ScriptedHttp`, `OfflineHttp` | The three implementations (from docs/spec/net.md, high) |
| The error | Four request cases plus the offline refusal, each carrying its URL (from docs/spec/net.md, high) |

## Behaviour

### Requests

`Http` exposes one method, which fetches a URL and returns its body as bytes,
and a caller that wants a document parses the bytes itself [REQ-1801] (from
docs/spec/net.md, high). Every request carries a timeout, 30 seconds by default
[REQ-1802] (from docs/spec/net.md, high).

`RealHttp` accepts only a URL whose scheme is `https` [REQ-1803] (from
docs/spec/net.md, high), and `Http` follows a redirect only to an `https` URL
[REQ-1805] (from docs/spec/net.md, high). Only a response with status 200 is a
body [REQ-1804] (from docs/spec/net.md, high). A body is bounded at 64 MiB
[REQ-1806] (from docs/spec/net.md, high).

### Implementations

`RealHttp` performs the request [REQ-1812] (from docs/spec/net.md, high).
`ScriptedHttp` answers from a table of URL to response [REQ-1813] and records
the requests it was given, in order [REQ-1903] (from docs/spec/net.md, high).
`OfflineHttp` fails every request with the offline error [REQ-1814] and makes no
request [REQ-1905] (from docs/spec/net.md, high).

### Errors

The error tells apart four cases a request can reach: the URL was refused before
any request, the host couldn't be reached, the server answered with a status,
and the body couldn't be read [REQ-1810] (from docs/spec/net.md, high). The
offline refusal is a fifth kind, outside those four, because no request happened
(from docs/spec/net.md, high). Every error carries the URL it is about
[REQ-1811] (from docs/spec/net.md, high).

## Failure paths

| Condition | What happens |
| --- | --- |
| The URL's scheme isn't `https` | `RealHttp` refuses it and says so, without attempting the request [REQ-1803] [REQ-1900] (from docs/spec/net.md, high) |
| A redirect points to a URL that isn't `https` | `Http` doesn't follow it [REQ-1805] (from docs/spec/net.md, high) |
| The host can't be reached | The request fails with the unreachable case and the URL [REQ-1810] [REQ-1811] (from docs/spec/net.md, high) |
| The status isn't 200 | The request fails, and the error carries the code [REQ-1804] [REQ-1901] (from docs/spec/net.md, high) |
| The body exceeds the bound | The request fails; the body isn't truncated [REQ-1902] (from docs/spec/net.md, high) |
| The body can't be read | The request fails with the unreadable-body case and the URL [REQ-1810] [REQ-1811] (from docs/spec/net.md, high) |
| The request runs past its timeout | The request fails with the unreachable case, or with the unreadable-body case when the body had started, naming the URL and saying that it timed out [REQ-1802] [REQ-1810] (from crates/meowctl-net/src/real.rs:79-88 and real.rs:118-121, medium) |
| `ScriptedHttp` gets a URL not in its table | It fails the URL in place of returning empty bytes [REQ-1904] (from docs/spec/net.md, high) |
| Any request reaches `OfflineHttp` | It fails with the offline error [REQ-1814] (from docs/spec/net.md, high) |
