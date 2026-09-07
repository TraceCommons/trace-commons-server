# Listing more than two coding tools — findings

Investigation for `docs/superpowers/specs/2026-09-07-harness-routing-design.md`.
Nothing implemented. Read against the IronWire revision this workspace actually
builds, `90c9ff946ee424977f7a7d8a97440264559fddd4` (`Cargo.lock`); the files
below are byte-identical between that checkout and `ed53375`, so both revisions
give the same answers.

## 1. Where a catalog comes from

Established from source:

- The daemon fetches it. `crates/ironwire_proxy/src/embed/catalog.rs` pins
  `const CATALOG_URL: &str = "https://ironwire.dev/releases/catalog.json"`,
  first check 60 s after start, then every 6 h, gated on the same
  `updates.check` switch as the release check (default `true`,
  `ironwire_core/src/config.rs`).
- It is cached at `<ironwire home>/catalog.json` (`PathsConfig::catalog_file`)
  and re-verified on every load, so a tampered cache is refused rather than
  trusted.
- `Catalog` is a plain value with public fields (`schema_version`, `serial`,
  `issued_at`, `anthropic`, `client_identity`, `models`, `agents`).
  `Catalog::default()` ships `agents: Vec::new()`, and a test asserts
  "the compiled-in default ships no agents".
- With no catalog, everything still works and `tools::all` returns exactly
  `["claude", "codex"]` — its own test says so. Failure is always onto the
  compiled-in defaults, never onto nothing.

## 2. Does a catalog naming Gemini or Cline exist today

No, and the channel is inert:

- `ironwire.dev` does not resolve. `dig ironwire.dev NS` returns
  `status: NXDOMAIN`; `curl` fails with "Could not resolve host" from a shell
  that reaches `github.com` fine. Every refresh therefore fails and is logged
  at debug.
- Even a reachable document could not be applied. `CATALOG_PUBLIC_KEY` is
  `[0u8; 32]`, documented as "deliberately **not** a usable key. Until release
  signing exists, every document fails verification". `docs/UPDATES.md` repeats
  it: "**Today the constant is a placeholder that cannot verify anything**".
- `docs/UPDATES.md`'s "what it carries today" table does not mention `agents`
  at all.

So depending on the fetched catalog is depending on something that is not
merely empty but cannot be non-empty without an upstream release.

## 3. What an `AgentEntry` can say

`ironwire_catalog::schema::AgentEntry` = `id`, `name`, `enabled`, `detect`
(executable **names**, no separators), `config: ConfigLocation`, and
`settings: Vec<AgentSetting>`. The constraints are the security argument for
letting a document introduce a tool at all:

- `ConfigLocation` is one or two path segments under `$HOME`, the first a
  dotdir, `.`/`..` refused, and the file must end `.json` or `.toml`.
- `AgentSetting` is a dotted key plus a `Facade`, which is `Anthropic` or
  `OpenAi` and nothing else. **There is no literal-value variant**: the URL is
  always `http://127.0.0.1:{port}/anthropic` or `.../openai`, computed by
  `Facade::url` from this binary's own port.

Three consequences for our four tools:

- **Gemini CLI cannot be described at all.** Its config location is fine
  (`~/.gemini/settings.json` exists on this machine), but IronWire serves
  exactly two façades — `server.rs` nests `/anthropic` and `/openai` — and
  neither is the Gemini wire shape. Writing our URL into a Gemini key would
  break the tool, not connect it. This is an upstream façade question, not a
  catalog question.
- **Cline is not described by anything we know.** Our own knowledge of Cline is
  transcript-only: `source/cline.rs` reads `~/.cline/data/sessions`. Nothing in
  this repo knows where Cline stores a provider setting, and `~/.cline` on this
  machine contains only `skills/`. Cline is a VS Code extension whose provider
  settings are believed to live in extension storage rather than a
  `.json`/`.toml` under a dotdir — *not established*, and it is exactly the
  thing to establish before promising anything.
- `source_copy.rs` knowing four tools is therefore **not** evidence that four
  tools can be connected. It knows how to *read* four tools' transcripts. That
  is a different fact from knowing how to point one at a listener, and the two
  should not be conflated in the UI.

Also worth stating plainly: a catalog entry can only fill keys with our façade
URL. Any tool that additionally needs a literal (a dummy API key, a provider
name, a model id) cannot be fully wired by a catalog entry — which is precisely
why Claude Code and Codex are hand-written code (statusline command, provider
table) rather than catalog rows.

## 4. The options

**(a) Consume the fetched signed catalog.** Free to write, worthless today: the
URL is NXDOMAIN and the key is zeros. Not ours to fix; both halves are upstream.

**(b) Contribute built-ins upstream to `nearai/ironwire`.** Correct for tools
whose wiring is more than one key — but for Gemini it needs a Gemini façade
first, and for Cline it needs a config writer for something that is not a JSON
or TOML file under a dotdir. Both are substantial upstream changes on someone
else's schedule. Not a release-blocker path.

**(c) Ship our own catalog value, in-process.** The important finding:
`tools::all`, `plan_connect` and `plan_disconnect` all take `&Catalog` as a
plain argument, and `Catalog`'s fields are public. **No signature is involved
in this path at all** — signing guards the network channel, not the type. We
can compile a `Catalog { agents: vec![…], ..Catalog::default() }` into our own
binary and hand it to `tools::all`. Cost: two new direct git dependencies at
the rev we already build (`ironwire_agents`, `ironwire_catalog` — today we
depend only on `ironwire_proxy`, which pulls them in transitively), plus a
table, plus per-tool verification. Risk: an entry that is wrong writes a wrong
key into someone's config — mitigated by `AgentEntry::problem()`, by the
preview, and by the three rules, but the *choice of key* is ours and nothing
validates it. It needs no upstream change and no third-party trust.

**(d) Ship two and say so.** Already largely done: `HARNESSES_WHAT` says "The
list is what this app knows how to look for, not every tool there is", and
`HARNESSES_NONE_FOUND` says what was looked for. Zero cost, and the only risk
is the unmet request.

## 5. Signature and trust

Verification is real and is the right shape: `SignedCatalog::verify` checks a
detached ed25519 signature over the exact stored bytes *before* deserialising,
`apply` refuses a serial at or below the installed one (rollback guard), a
newer schema is refused rather than half-applied, and any failure leaves the
previous document or the compiled-in defaults in force. The schema is
deliberately unable to name a host, a URL, or a path outside a dotdir.

The key is `CATALOG_PUBLIC_KEY`, compiled in, all zeros, held (when it is real)
by IronWire's release signing infrastructure — **not** by us. So option (a)
means trusting nearai's future release key to decide which files on a
contributor's machine we offer to edit; the schema bounds the damage to "one of
our own loopback URLs written into some dotdir `.json`/`.toml` of theirs", which
is a real bound but is not nothing. Option (c) takes on no third-party key at
all: the trust is our own release, which we already ask for.

## Recommendation

1. **Release with the two built-in tools and the honest copy that already
   exists.** Do not block on this. `HARNESSES_WHAT` already says what the list
   is; that sentence should be treated as load-bearing rather than as filler.
2. **Build the list against a `Catalog` value we own, not `Catalog::default()`,
   from day one** — even while that value is empty. That makes adding a tool a
   table edit rather than a re-plumbing, and costs nothing now.
3. **Add entries only per tool, per evidence**: config file is `.json`/`.toml`
   under a one- or two-segment dotdir; a base-URL-only key really redirects it;
   and it speaks the Anthropic or OpenAI wire shape. Gemini CLI fails the third
   test today and Cline fails the first as far as anything here establishes, so
   neither is a candidate for the first entry.
4. **Do not enable the fetched catalog** and do not present it as the route to
   more tools.

## Side finding

Because `updates.check` defaults to `true` and this repo never writes an
IronWire `config.toml`, the embedded daemon spawns a refresh task that resolves
`ironwire.dev` — NXDOMAIN — 60 s after start and every 6 h thereafter, forever.
Harmless, but it is a periodic outbound DNS lookup for a domain that does not
exist, made by an app whose whole pitch is about where calls go. Worth a
follow-up decision rather than a silent default.

## Provenance

Established from source or from a command run here: everything in §1, §2, §3,
§5, the façade count, the `Catalog` field visibility in §4(c), and the side
finding. Inference, and labelled as such above: that Cline keeps its provider
settings in VS Code extension storage; that opencode-style tools would be
representable (the location rules allow `~/.config/<tool>/x.json`, but no key
has been verified); and the judgement calls in the recommendation.
