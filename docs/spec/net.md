# The network

**Crate:** `meowctl-net`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §3
**v0.1.0 equivalent:** the `*http.Client` fields on `RegistryLoader`,
`githubLoader`, and `ctx.download`

## Scope

This component owns every request meowctl makes, behind one trait. It is the
third effect, beside [the filesystem](fs.md) and [processes](exec.md), and it
is a trait for the same two reasons those are: a test must be able to answer a
request without a server, and a caller must not be able to reach the network by
some other route.

It knows nothing about modules, registries, or tarballs. It fetches bytes from
a URL and says why it could not.

## Boundary

The `Http` trait, its implementations, and the error a failed request produces.

## Requests

**[R-NET-001]** `Http` MUST expose one method: fetch a URL and return its body
as bytes. A component that wants a document parses the bytes itself, because a
trait that decoded JSON would need a second method for TOML and a third for a
tarball.

**[R-NET-002]** A request MUST carry a timeout, defaulting to 30 seconds, which
is what `v0.1.0` gives every one of its clients. A shell hook that triggers a
resolution is otherwise a hang with no output.

**[R-NET-003]** `RealHttp` MUST refuse a URL whose scheme is not `https`, and
MUST say so rather than attempting the request. `v0.1.0` accepts whatever
string the index hands it, so a registry index could downgrade a module fetch
to plaintext by writing one `source` template; see [R-MODULE-013].

**[R-NET-004]** A response whose status is not 200 MUST fail, and the error
MUST carry the code. `v0.1.0` does this and the code is what tells a missing
module from a rate-limited one.

**[R-NET-005]** `Http` MUST NOT follow a redirect to a non-`https` URL. A
redirect is the other way a plaintext fetch happens, and it is not visible in
the URL a caller passed.

**[R-NET-006]** A response body MUST be bounded, and a body that exceeds the
bound MUST fail rather than being truncated. `v0.1.0` calls `io.ReadAll` on
every response, so a server that streams forever is an out-of-memory kill with
no message. The bound is a number rather than a judgement, and 64 MiB is two
orders of magnitude above the largest module published.

## Failures

**[R-NET-010]** The error MUST distinguish four cases that a request can reach:
the URL was refused before any request, the host could not be reached, the
server answered with a status, and the body could not be read. A fifth, the
refusal [R-NET-014] produces, is not one of them because no request happened. [R-MODULE-060] requires a module
resolution to say which of these happened, and it can only say it if this trait
reports it.

**[R-NET-011]** An error MUST carry the URL it is about. A resolution fetches an
index, a tarball, and a commit; an error that says only "connection refused"
does not say which.

## Implementations

**[R-NET-012]** `RealHttp` MUST perform the request.

**[R-NET-013]** `ScriptedHttp` MUST answer from a table of URL to response, MUST
record the requests it was given in order, and MUST fail a URL that is not in
the table rather than returning empty bytes. A test that silently gets nothing
back passes for the wrong reason.

**[R-NET-014]** `OfflineHttp` MUST fail every request with the offline error and
MUST make no request. It is how [R-MODULE-043] is proved: a resolution that
completes against `OfflineHttp` made no network call, and no assertion about
call counts can be forgotten.

## Parity with v0.1.0

The 30-second timeout, the status check, and the error text shape come from
`internal/starlark/loader/registry.go`.

[R-NET-003] and [R-NET-005] are new. `v0.1.0` builds its requests with
`http.NewRequestWithContext` and the default client, which follows redirects
and speaks whatever scheme the URL names. Neither has caused a problem, and
neither is something a reader of a registry index can check for themselves,
which is why they are stated here rather than left to the caller.
