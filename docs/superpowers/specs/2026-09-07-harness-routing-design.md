# Per-harness routing in the Model calls destination — design

## Why

The Model calls destination has one switch: whether this computer answers model
calls at all. It says nothing about *which tools send calls here*, and there is
no way to find out from the app.

The offer copy already promises the missing half:

> Turning this on does not change where any tool sends its calls. That stays a
> separate choice, **made one tool at a time**.
> — `OFFER_NO_REPOINT`

That per-tool choice is real, but today it is made by hand, in each tool's own
config file, outside this application, with no way to see the result. A
contributor who turns the switch on and expects something to happen sees
nothing — which is the same failure that put this feature on a buried settings
toggle in the first place.

**This slice gates the next release.** The destination alone is not a
releasable state.

## What already exists — the important finding

Almost none of this needs building. IronWire ships a crate,
`ironwire_agents`, whose entire purpose is this problem:

> Pointing a coding agent's own config file at IronWire, without taking it over.
> [...] the control API has to answer "which tools does this machine have, and
> are they pointed at us" as well.

It already carries the safety rules that make editing a file we do not own
acceptable at all, quoted from its own header:

- **Never rewrite a file we cannot parse.** A user's own syntax error must not
  come back looking like ours.
- **Fill an empty slot; leave a full one alone.** A value already in the key is
  another proxy or a deliberate choice, and taking it over would move someone's
  traffic without telling them. It is reported, not overwritten.
- **Remove only what we put there.**

The API is a close fit for the surface we want:

- `tools::all(&Catalog) -> Vec<Tool>`, where `Tool` is `{ id, name,
  config_path: Option<PathBuf>, installed: bool, wired: bool, connect_command:
  String }`. `wired` is precisely "whether its config currently sends calls
  here".
- `tools::plan_connect(id, port, &Catalog) -> Result<Planned, Error>` and
  `tools::plan_disconnect(id, &Catalog)`.
- `Planned { path, changes: Vec<String>, occupied: Vec<(String, String)> }` with
  `is_noop()`. `changes` is already phrased in words, and `occupied` names the
  slots it refused to take over.
- `tools::commit(&Planned) -> io::Result<Option<PathBuf>>`.

Two tools are built in — `claude` ("Claude Code") and `codex` ("Codex"). Anything
further arrives through a **signed catalog** (`ironwire_catalog::schema::AgentEntry`).

## A correction to the chosen mechanism

The design was chosen as: adopt a harness once by writing its config, then
toggle serve/pass-through **inside** IronWire, so ordinary flipping never
touches a file.

**That switch does not exist.** IronWire's control API is only
`/_ironwire/health` and `/_ironwire/status`, and there is no per-tool
enable, disable or pass-through anywhere in `ironwire_proxy` or
`ironwire_core`. The only mechanism is connect/disconnect, and both are config
writes.

So this spec is written against connect/disconnect. That is a smaller loss than
it first appears, because the three rules above already remove the hazards that
made a per-toggle config write look risky:

- Disconnect needs no saved original — it removes only what IronWire put there.
- Connect cannot clobber a value the contributor set — an occupied slot is
  reported, never overwritten.
- An unparseable file is refused rather than rewritten.

What is genuinely lost is speed and reversibility: each flip is file I/O against
a tool that may rewrite its own config, rather than an in-memory state change.
If that proves annoying in use, the fix is an upstream IronWire capability —
a per-tool pass-through in the control API — and this design should be revisited
then rather than worked around here.

## Scope

**In:** listing the harnesses in the Model calls destination with their real
state; connecting and disconnecting one at a time, with a preview of exactly what
would change; showing which need restarting.

**Out:** any change to the master switch's behaviour; the global hotkey (still
cut 2); routing decisions per model or per project; anything that writes more
than one tool's config in a single action.

## Design

### 1. The list

The destination gains a section listing every harness `tools::all` reports, each
showing: its name, whether it is installed, whether it currently sends calls
here, and the config file that would be edited. The file path is shown always,
not on demand — a tool nobody expected to be configured is a question about
*which file*, every time.

A harness that is not installed is listed and disabled rather than hidden.
Hiding it makes the absence of a tool indistinguishable from the app not knowing
about it.

When no catalog is present, only the two built-in tools appear. The surface must
say that the list is what this machine knows about, not a claim about every tool
that exists.

### 2. Connect and disconnect are previewed, never immediate

A toggle calls `plan_connect` / `plan_disconnect` and shows the resulting
`Planned` before anything is written:

- `changes` — what would change, in IronWire's own words.
- `occupied` — slots left alone because the contributor is already using them.
  This is the case where the honest answer is "we did not take that over", and
  it must be shown, not swallowed.
- `is_noop()` — nothing to do; say so rather than showing an empty confirmation.

Only on confirmation does `commit` run. **We are editing a file we do not own;
the contributor sees the diff first.** This mirrors the reason the master switch
became a destination: the consequence is stated where the decision is made.

### 3. Restarting

A harness holding an old setting in a running process is shown as needing a
restart, so the list never claims a tool is sending calls here when the process
in front of the contributor is not. This is the same rule as the tone: the
surface reports what is true, not what was asked for.

### 4. Wording

Every string comes from `private_inference_copy.rs`, and the banned-word sweep
applies with full force. It rejects **`route`**, `proxy`, `backend`, `endpoint`,
`localhost`, `private`, `secure`, `encrypt`, `anonym`, `protect`, `credit`,
`earn`, and vendor names.

So this surface may not say "routes through the proxy", "points at the backend",
or "the local endpoint". It says what happens instead: a tool **sends its calls
here**, and this computer **answers** them. Tool names themselves ("Claude
Code", "Codex") come from `Tool.name`, which is IronWire's, not ours to
restate.

`connect_command` is a command string, not prose, and is shown verbatim as a
fallback for a contributor who would rather do it themselves.

## Testing

- A harness whose slot is occupied is reported as occupied and **not**
  overwritten — the rule most likely to be broken by a well-meaning
  simplification.
- An unparseable config is refused, and the refusal is distinguishable from
  "nothing to change".
- `is_noop()` produces a distinct message rather than an empty preview.
- Disconnect removes only what IronWire wrote, leaving neighbouring keys intact.
- A tool that is not installed cannot be connected.
- The list degrades to the two built-in tools with no catalog, and says so.
- No shell authors a harness string; every sentence resolves to the copy module.
- The banned words, including `route`, appear nowhere in the new copy.

## Risks

- **Editing files we do not own.** The main hazard, and the reason preview
  precedes commit. IronWire's three rules do the heavy lifting; this design
  must not route around them for convenience.
- **A tool rewriting its own config underneath us.** `wired` is read each time
  the list is shown rather than cached, so the surface corrects itself.
- **Catalog absence read as tool absence.** Mitigated by saying what the list
  is.
- **Per-flip file I/O.** Accepted, with the upstream fix named above.
