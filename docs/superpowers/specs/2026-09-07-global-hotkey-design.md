# A key combination that works anywhere — design (cut 2)

Cut 2 of `2026-09-07-private-inference-top-level-design.md`. Cut 1 shipped the
model-calls destination, the tray/menu-bar section, and **in-app** shortcuts.
This is the deferred piece: a combination that fires while the contributor is in
their editor, with the app unfocused.

Everything below that is stated as fact was checked on this machine or read out
of a primary source, and every such claim carries its source. Everything that is
inference is labelled as inference. A short section at the end lists what could
**not** be established here.

---

## 1. The rule this inherits

From `tray.rs`, `TrayIcon.cs` and `MenuBarView.swift`, already shipped:

> The menu can stop this computer answering model calls, and it cannot start it.
> Turning it OFF only ever reduces what this computer will answer, so it is safe
> to press with nothing else on screen, while turning it ON changes what anything
> else running here may send through, charged to the contributor's own accounts.

A global combination is the most off-screen surface this application will ever
have — no icon, no menu, no window, nothing on screen at the moment it fires. So
the rule applies with more force here than it does to a tray, not less.

**The combination does exactly one thing, and there is no toggle.**

| Switch position when pressed | What the combination does |
| --- | --- |
| Answering | Stops answering. No window, no confirmation. |
| Not answering | Raises the window at the model-calls destination and does nothing else. |

That is not a compromise, it is the whole shape. It also removes the worst
failure mode of a blind toggle: a contributor who cannot remember which way the
switch was pointing cannot flip it the wrong way by pressing a key. The press is
idempotent in the safe direction.

Consequence for the code: the platform layer raises the same two requests the
tray already has — `TrayRequest::StopAnsweringModelCalls` / `Open(screen)` on
GTK, `PrivateInferenceStopRequested` / `PrivateInferenceRequested` on Windows.
**No new request type is introduced, and no hotkey handler calls the daemon
directly.** The existing surfaces are the specification for this one.

---

## 2. macOS

### 2.1 The app's signing posture (established)

`macos/scripts/make-release-dmg.sh:197-204`:

```
# Hardened runtime is required for notarization. There is deliberately no
# entitlements file: this app needs no exception to the hardened runtime, and
# adding entitlements it does not use would widen what a compromised process
# could do for no benefit.
```

So, established from the repo:

- **Not sandboxed.** There is no entitlements file at all, therefore no
  `com.apple.security.app-sandbox`. This is a Developer ID / DMG / notarised
  app, not a Mac App Store one.
- **Hardened runtime is on** (`codesign --options runtime`).
- `macos/scripts/info-plist.sh` declares no TCC usage-description key of any
  kind, and none is currently needed.

### 2.2 What a global combination actually requires (established by probe)

There are two mechanisms, and they have completely different permission stories.

**(a) `RegisterEventHotKey` — no permission, no prompt, no entitlement.**

Declared in the macOS 26 SDK at
`.../HIToolbox.framework/Versions/A/Headers/CarbonEvents.h:15484`, still exported
from `HIToolbox.tbd`. A throwaway Swift tool built with Xcode 26.6 against
`MacOSX26.5.sdk` on macOS 26.6.2, **unsigned, not in a bundle**, registered
`Cmd-Shift-Opt-M` and got:

```
RegisterEventHotKey status = 0
```

No prompt appeared, no entitlement was present, hardened runtime was not
involved. The header documents no permission requirement and there is no TCC
class for it. This is the mechanism `MASShortcut`, `KeyboardShortcuts` and every
other Mac hotkey library uses.

*Caveat, stated plainly:* the probe ran as a child of a terminal that already
holds Accessibility and Input Monitoring grants (`AXIsProcessTrusted = true`,
`CGPreflightListenEventAccess = true` in the same run). Registration returning
`noErr` is therefore proof that **registration** needs nothing, but this machine
could not be used to prove that **delivery** of `kEventHotKeyPressed` still works
in a process with those grants absent. Documentation and universal practice say
it does — hot keys are dispatched by the window server, not by an event tap — but
that last step is inference here, not measurement. It should be confirmed once on
a fresh account before shipping (see §8).

**(b) `NSEvent.addGlobalMonitorForEvents` — needs Accessibility. Do not use it.**

From the SDK header, `AppKit.framework/.../NSEvent.h:541`:

> Key-related events may only be monitored if accessibility is enabled or if your
> application is trusted for accessibility access (see `AXIsProcessTrusted`) …
> you can only observe the event; you cannot modify or otherwise prevent the
> event from being delivered to its original target application.

Two disqualifiers at once. It needs the TCC grant, and it cannot consume the
key, so the contributor's editor would also receive the keystroke. **The design
does not use global monitors or `CGEventTap`.** The permission investigation that
cut 1 anticipated turns out not to apply to the mechanism we should pick.

### 2.3 Refused vs never-asked (established: not distinguishable)

This matters only if we were to take route (b), but it is worth recording because
it is the thing that is easiest to misremember. The public APIs return one bit:

- `CGPreflightListenEventAccess()` — SDK comment: *"Checks whether the current
  process already has event listening access"*
  (`CoreGraphics.framework/Headers/CGEvent.h:399`).
- `AXIsProcessTrustedWithOptions()` — same shape.

Neither has a tri-state. "Denied" and "never asked" are the same `false`, and the
authoritative record is `TCC.db`, which is SIP-protected. **A design that needs to
tell those apart cannot get it from a supported API.** Another reason (b) is out.

### 2.4 Collision detection (established: effectively none)

The header documents non-exclusive registration by default — *"The same hot key
can, however, be registered by multiple applications"* — and
`kEventHotKeyExclusive` (10.5+), which is documented to return
`eventHotKeyExistsErr` when another process already holds the same chord
exclusively.

A second throwaway probe registered nine chords **exclusively**, including ones
this machine demonstrably has bound:

```
exclusive Cmd-Space (Spotlight):       status=0
exclusive Ctrl-Space (input source):   status=0
exclusive Cmd-Shift-M:                 status=0
exclusive Cmd-Shift-Opt-M:             status=0
exclusive Ctrl-Opt-M:                  status=0
exclusive Ctrl-Shift-Cmd-M:            status=0
exclusive Cmd-Shift-Opt-K:             status=0
exclusive Ctrl-Opt-Cmd-Space:          status=0
exclusive Cmd-Q (Quit):                status=0
```

Every one succeeded, including `Cmd-Space` and `Cmd-Q`. So:

**On macOS the system will not tell us whether a combination is free.** System
shortcuts live in `com.apple.symbolichotkeys` (the user's own overrides are
readable with `defaults read com.apple.symbolichotkeys`, and on this machine that
plist holds only nineteen `enabled` entries, i.e. deviations, not the full set);
app shortcuts are menu items, invisible to us entirely. There is no supported
enumeration. §5 is written around this.

### 2.5 Where the handler lives

`RegisterEventHotKey` targets `GetApplicationEventTarget()` and delivers
`kEventHotKeyPressed` through an `InstallEventHandler`, which runs under the
Cocoa run loop that `TraceCommonsShell` already has. No second binary, no helper
process, no `LSUIElement` change. The one constraint is that the header says the
API is *"Not thread safe"* — register and unregister on the main actor only.

---

## 3. Windows

### 3.1 The message loop already exists (established)

`windows/src/TraceCommons.App/TrayIcon.cs:303-333` already creates a private
`WS_POPUP` window with its own `WndProc`, deliberately not subclassed into the
WinUI window:

> WinUI owns its message handling, and inserting a subclass into it to catch one
> custom message is a borrowed-authority bug waiting to happen.

That window is the correct `hWnd` for `RegisterHotKey`, and `OnMessage` is the
correct place to add a `WM_HOTKEY` (`0x0312`) case beside the existing
`CallbackMessage` case. **Nothing new is created.** The class already raises
`PrivateInferenceStopRequested` and `PrivateInferenceRequested`, and its own
XML doc already states the one-direction rule; the hotkey raises the same two
events and adds no third.

### 3.2 `RegisterHotKey` semantics (established from Microsoft docs)

From `learn.microsoft.com/.../nf-winuser-registerhotkey`:

- Passing an `hWnd` posts `WM_HOTKEY` to that window's queue. Passing `NULL`
  posts to the calling thread's queue. We pass the tray window's `hWnd`.
- Return value: nonzero on success, **zero on failure**, with
  `GetLastError` for detail.
- *"Typically, RegisterHotKey also fails if the keystrokes specified for the hot
  key have already been registered for another hot key. However, some
  pre-existing, default hotkeys registered by the OS (such as PrintScreen …) may
  be overridden."*
- `MOD_NOREPEAT` (`0x4000`) suppresses auto-repeat. **Use it.** Without it, a
  held key would fire "stop answering" dozens of times.
- *"Keyboard shortcuts that involve the WINDOWS key are reserved for use by the
  operating system."* So no `MOD_WIN`.
- F12 is reserved for the debugger at all times.
- The id must be in `0x0000`–`0xBFFF` for an application.

**So Windows is the one platform where a collision is detectable** — the `FALSE`
return is a real signal. Two caveats:

1. Design instruction: **branch on the return value, not on a specific
   `GetLastError` code.** `ERROR_HOTKEY_ALREADY_REGISTERED` (1409) is the usual
   value but the documentation does not name it, and this machine is not Windows,
   so it was not verified. Treating any failure as "taken" is correct and needs no
   unverified constant.
2. It detects collisions only against other `RegisterHotKey` callers. An editor's
   own `Ctrl+Shift+M` is an in-app accelerator, invisible to this check, and a
   successful `RegisterHotKey` **silently takes that chord away from the editor
   while this app runs**. That is the same hazard macOS has; Windows just also
   catches the smaller, tractable case.

### 3.3 The chord this app already spends

`MainWindow.xaml:47` binds `Ctrl+Shift+M` as the in-app toggle, and the comment
above it says exactly what this document must respect:

> A global hot key takes a chord away from every other application on the machine
> whether or not this app is running, which was cut deliberately and is not to be
> reinstated here.

Note the interaction: if the global registration used the same `Ctrl+Shift+M`, it
would preempt the accelerator **inside our own window too**, and the two have
different semantics (in-app toggles, global only stops). That would be a genuine
bug. The global combination must therefore differ from the in-app one, or the
in-app accelerator must be removed while a global one is bound. §5 resolves this
by making the global one a different, contributor-chosen chord.

---

## 4. GTK / Linux — the hard one

### 4.1 What `portal.rs` establishes

`crates/trace-commons-contributor-gtk/src/portal.rs` is the pattern, and it is
better than anything this design would have invented. It already:

- treats "no portal, or no backend for this interface" as **ordinary, not a
  failure** (module doc, lines 5-8);
- classifies its own outcome into `BackendState::{Present, Absent, Unknown}`
  rather than only logging, because *"a silent no-op on a desktop with no
  `Background` backend is exactly the failure mode this product refuses
  everywhere else"*;
- pins the exact D-Bus error names that mean "nothing answers for this"
  (`UnknownMethod`, `UnknownInterface`, `ServiceUnknown`, `NameHasNoOwner`) and
  has tests for each;
- bounds the wait (`PROBE_TIMEOUT = 5s`) so the UI never hangs on a portal that
  never answers;
- **never guesses.** A non-D-Bus failure is `Unknown`, never `Absent`.

A `GlobalShortcuts` probe would be the same shape against a different interface,
and would reuse `is_absent_backend_error` verbatim.

### 4.2 Is there a portal? Yes — and its shape is better than ours

`org.freedesktop.portal.GlobalShortcuts`, version 2
(flatpak.github.io/xdg-desktop-portal). `CreateSession` → `BindShortcuts` →
`Activated` / `Deactivated` / `ShortcutsChanged` signals. `ListShortcuts` reads
back what is bound.

The important property: **the application does not choose the key combination.**
`BindShortcuts` presents a dialog and the compositor and the user decide;
`preferred_trigger` is an optional hint only. Which means the entire §5 problem —
"what chord is actually free?" — does not exist on this path. The desktop owns
the keymap, knows what is taken, and asks the contributor. That is the correct
answer, and neither macOS nor Windows offers it.

The flatpak manifest already grants `--socket=session-bus`
(`crates/trace-commons-contributor-gtk/flatpak/ai.tracecommons.Contributor.yml:62`),
which is the unrestricted branch, so a `GlobalShortcuts` call needs **no new
manifest permission**. Established by reading the manifest and its own comment.

### 4.3 Where it is not there

Backend coverage is the problem, and it is unsettled:

- KDE Plasma implements it (the portal's reference implementer).
- GNOME added it in `xdg-desktop-portal-gnome` for GNOME 48.
- A third-party field report
  (`aaddrick/claude-desktop-debian`, `docs/learnings/wayland-global-shortcuts-portal.md`)
  states that **wlroots compositors — Sway, Hyprland, Niri — lack it**, that
  binding fails there, and that GNOME 50 has an upstream blocker; it also reports
  that mutter on GNOME ≥ 49 no longer honours XWayland-side global key grabs, so
  an X11 fallback only fires when the window already has focus, i.e. it is not
  global at all. **This is a third-party report, not a primary source, and it was
  not reproduced here.** No Linux desktop was available on this machine.
- X11 without the portal would mean an `XGrabKey` path. That is a second
  mechanism with its own failure modes on a machine that may be running either
  session type, and it is the kind of thing that works in testing and silently
  stops working on the reviewer's laptop.

### 4.4 What `ui/mod.rs` requires

> GNOME has no system tray, so nothing here may depend on one: every capability
> is reachable from this window, and the tray — where a desktop has a real one —
> would only ever be a shortcut into it. Nothing in this application tells a
> contributor to install a shell extension.

And `tray.rs`: *"a bonus, never a foundation."*

A global combination is a shortcut into a capability, not a capability, so the
rule permits it as a bonus. But the same rule forbids three things:

1. advertising it where it does not exist;
2. telling the contributor to install or change anything to get it;
3. any silent no-op — a bound-looking combination that never fires is worse than
   an absent one, and it is exactly what `portal.rs` exists to prevent.

Which means the Linux path is buildable but it is `portal.rs`-sized work: a
session that must be created and re-created across portal restarts, a
`ShortcutsChanged` subscription, a `ListShortcuts` read-back so the destination
can show the combination the desktop actually assigned, and an honest `Absent`
state that says so.

---

## 5. Chord choice — and why the design does not pick one

### How availability was determined

- **macOS:** measured. Exclusive registration succeeds for every chord tried,
  including `Cmd-Space` and `Cmd-Q` (§2.4). The system will not answer the
  question. `com.apple.symbolichotkeys` holds only the user's deviations from
  the defaults, and app-level shortcuts are menu items we cannot see at all.
- **Windows:** partially answerable — `RegisterHotKey` fails against another
  `RegisterHotKey` caller (§3.2) and against nothing else. In-app accelerators in
  editors are invisible to it.
- **Linux:** the compositor answers it, because the compositor is the one
  choosing (§4.2).

So on two of three platforms **there is no way to establish that a chord is
free**, and the request's own standard — "say how you determined availability
rather than asserting it" — cannot be met for any fixed chord. A shipped default
of `Ctrl-Shift-Opt-M` would be an assertion, and the first contributor whose
editor binds it would lose that binding with no error anywhere.

### The conclusion that follows

**No combination is bound by default. The feature ships off.**

- The destination shows a "no combination is set" row and a recorder.
- The contributor presses the chord they want; the app records it and tries to
  bind it.
- On Windows a failed `RegisterHotKey` is reported at once and nothing is stored.
- On macOS nothing can be reported, so the recorder's copy says what the system
  cannot tell us, and clearing it is one press away (§6).
- On Linux the recorder is replaced by the portal's own dialog and a read-back
  of what the desktop assigned.

This also disposes of the §3.3 conflict: the in-app `Cmd/Ctrl-Shift-M` keeps its
meaning, and a contributor who binds a global one picks something else. If they
pick the same chord, the app refuses it and says so — that check we *can* make,
because it is our own binding.

Shipping unbound is a real cost: a feature nobody turns on is a feature that did
not ship. It is accepted here because the alternative is silently confiscating a
keystroke from an unknown editor, and the exposure this feature governs is the
one place in this product where a surprise is least acceptable.

---

## 6. Discoverability and revocation

A global combination the contributor cannot see or disable is a defect. Four
requirements, all satisfied on the destination that cut 1 built:

1. **It is set from the model-calls destination and nowhere else.** Same screen
   as the switch and `OFFER_EXPOSURE`. There is no tray item, no first-run offer
   and no notification that sets a combination — the on-direction rule covers
   setting it too, because binding a key is a decision about this machine.
2. **The destination always shows the current combination as text**, read back
   from what is actually bound (on Linux, from `ListShortcuts` — never from what
   we asked for), so the app cannot claim a binding it does not hold. This is the
   same discipline as the tone rule: the switch reports what was asked for, the
   indicator reports what is true.
3. **Clearing is one control, always present, and never fails.** Unbinding is in
   the safe direction, so it needs nothing that binding needs.
4. **It survives nothing.** Quitting the app releases the chord on all three
   platforms (`UnregisterEventHotKey` is not even required at exit — the header
   says *"the system will take care of that for you"*), so an uninstalled or
   quit app never holds a key hostage.

### Copy

Every string goes in `crates/trace-commons-contributor/src/private_inference_copy.rs`,
**between the `PRIVATE-INFERENCE-SURFACE-*` markers**, or the sweep at
`private_inference_copy.rs:1010-1072` will not see it — the sweep splits the
marked region and asserts `strings.len() >= 18`, so a constant added outside the
markers is silently unswept. Banned: `ironwire`, `iron wire`, `proxy`, `backend`,
`route`, `endpoint`, `localhost`, `private`, `secure`, `encrypt`, `anonym`,
`protect`, `credit`, `earn`.

Drafts, each checked against that list (note "combination", not "shortcut key" or
"hotkey" — the plain word survives translation to all three shells and says what
it is):

| Constant | Text |
| --- | --- |
| `HOTKEY_TITLE` | `A key combination that works anywhere` |
| `HOTKEY_WHAT` | `Set a combination here and it will stop this computer answering model calls, from whatever you happen to be working in.` |
| `HOTKEY_ONE_DIRECTION` | `It can only stop. When this computer is not answering, the combination brings this window up here instead, so the sentence above is on screen before anything starts.` |
| `HOTKEY_UNSET` | `No combination is set.` |
| `HOTKEY_SET_PREFIX` | `Set to` (followed by the combination, rendered by the shell from what is bound) |
| `HOTKEY_RECORD` | `Choose a combination` |
| `HOTKEY_CLEAR` | `Use no combination` |
| `HOTKEY_TAKEN` | `Something else on this computer is already using that combination. Choose a different one.` |
| `HOTKEY_UNVERIFIABLE` | `This computer does not report whether a combination is already in use, so a combination another app also uses may reach one of you and not the other. If it does nothing, choose a different one.` |
| `HOTKEY_SAME_AS_IN_APP` | `That combination already does something in this window. Choose a different one.` |
| `HOTKEY_DESKTOP_CHOOSES` | `Your desktop chooses this combination and asks you for it.` |
| `HOTKEY_NOT_OFFERED_HERE` | `This desktop does not offer combinations to apps like this one. Everything here is still reachable from this window.` |

`HOTKEY_TAKEN` is Windows-only in practice; `HOTKEY_UNVERIFIABLE` is macOS-only;
the last two are Linux-only. All four live in the shared module anyway, because
the module is where the wording is decided, not where the platform is.

---

## 7. Recommendation

**Build it on macOS and Windows. Offer nothing on Linux in this cut.**

### macOS — build. Cheapest of the three, and the finding is the opposite of what cut 1 assumed.

Cut 1 deferred this because it expected "macOS Accessibility / Input Monitoring
with a user-facing prompt". Measured, that is wrong for the mechanism we should
use: `RegisterEventHotKey` needs **no TCC grant, no prompt, no entitlement, and
no change to the hardened-runtime or notarisation setup** (§2.2). There is no
permission-explanation copy to write and no refused-vs-never-asked problem to
solve, because we never ask.

Cost: a `GlobalHotKey` type in `TCShellCore` (register/unregister,
`InstallEventHandler`, main-actor only), a recorder control on the destination,
persistence of the recorded chord, and the copy. No packaging change. Call it
small — comparable to the menu-bar section cut 1 already landed. The real cost is
the recorder UI, not the hotkey.

Residual risk: no collision detection at all, mitigated by shipping unbound and
by `HOTKEY_UNVERIFIABLE`.

### Windows — build. Slightly more code, better behaviour.

The message loop, the hidden `HWND`, and both request events already exist
(§3.1). The addition is a `WM_HOTKEY` case, `RegisterHotKey`/`UnregisterHotKey`
with `MOD_NOREPEAT`, and the same recorder. It is the only platform that can tell
the contributor "that one is taken", which makes it the best of the three
experiences.

Cost: small, and lower than macOS because the failure path is real rather than
hedged. One caution — this reverses the explicit comment at `MainWindow.xaml:36`,
which says a global hot key "was cut deliberately and is not to be reinstated
here." That comment should be rewritten in the same commit to say what is now
true: the accelerators there stay in-app, and the global one is a separate,
unbound-by-default, off-only binding set from the destination. Leaving a comment
that contradicts the code is how the next reader concludes the feature is a
mistake.

### Linux — offer nothing now. Not "something unreliable".

Three reasons, in order of weight:

1. **Coverage is unsettled and getting worse, not better.** KDE has it, GNOME 48
   has it, wlroots compositors reportedly do not, and GNOME 50 reportedly has a
   blocker (§4.3, third-party). A feature whose availability depends on which
   GNOME point release the contributor is on is not a feature, it is a support
   burden.
2. **Linux does not have cut 1's shortcuts yet.** There is no
   `set_accels_for_action` anywhere in `crates/trace-commons-contributor-gtk/src`
   — grepped, zero hits outside one comment in `preview.rs`. macOS and Windows
   both shipped `Cmd/Ctrl-1..N`; GTK shipped none. Building a *global* shortcut
   for a shell that has no *in-app* shortcut is building the roof first. The
   correct next Linux commit is `Ctrl-1..4` and `Ctrl-Shift-M` in the window.
3. **The parity rule does not bind here.** Cut 1's "all three shells ship
   together" is about the destination and its copy, which are shared by
   construction. Cut 1's own text already conceded that this piece is
   "compositor- and portal-dependent behaviour on GTK". A mechanism that the
   platform genuinely does not provide is not drift.

When Linux is done — as its own cut, not squeezed into this one — the portal is
the only acceptable mechanism (no `XGrabKey` fallback), and it must follow
`portal.rs` exactly: probe, classify into `Present`/`Absent`/`Unknown`, bound the
wait, show `HOTKEY_NOT_OFFERED_HERE` on `Absent`, show nothing that claims a
binding on `Unknown`, and read the actual combination back with `ListShortcuts`.
Cost: medium — larger than macOS and Windows combined, because it is a session
lifecycle rather than a call.

---

## 8. Testing

- **The direction rule, per shell.** With the switch on, the combination raises
  the stop request and no other. With the switch off, it raises the open request
  and **no stop, no start**. Assert that no code path from a hotkey handler
  reaches a daemon write that turns model calls on. This is the regression that
  would otherwise ship silently, exactly as the tone rule was for cut 1.
- **Unbound by default.** A fresh profile has no combination and registers
  nothing at launch. Assert the registration call is not made.
- **Auto-repeat.** Windows: `MOD_NOREPEAT` is present in the flags passed.
- **Self-collision.** Recording the in-app chord is refused with
  `HOTKEY_SAME_AS_IN_APP`.
- **Copy parity and the sweep.** No shell contains a hotkey string literal; every
  new constant sits between the `PRIVATE-INFERENCE-SURFACE-*` markers so the
  banned-word sweep actually sees it. Bump the sweep's `>= 18` floor.
- **Read-back, not intent.** The destination's "set to" line is driven by what is
  bound, never by the stored preference. On Linux that means `ListShortcuts`.
- **macOS Swift tests run in CI** (`macOS app tests`, `swift test` on
  `macos-26`), so the direction assertions are enforced there.

---

## 9. What was not established here

Stated separately so none of it is read as fact:

- **Delivery without Accessibility.** Registration was measured to need nothing
  (§2.2). That `kEventHotKeyPressed` is still *delivered* in a process with no
  Accessibility and no Input Monitoring grant was **not** measured — the probe
  process inherited both. Confirm on a fresh macOS account before shipping.
- **Whether a Carbon hot key consumes the keystroke** rather than also passing it
  to the focused app. Widely-observed behaviour, and the basis for §5's "silently
  confiscating a keystroke" argument, but not tested here; it needs an
  interactive key press this session could not perform.
- **`ERROR_HOTKEY_ALREADY_REGISTERED` = 1409.** Recalled, not verified. The
  documentation names only the `FALSE` return, and the design deliberately
  branches on that instead.
- **Everything in §4.3 about wlroots and GNOME 50.** Third-party field report,
  not reproduced. No Linux desktop was available. Re-check against
  `xdg-desktop-portal-gnome` and each compositor before the Linux cut.
- **Any claim about what specific editors bind.** No method was found to
  enumerate an installed editor's default keymap, which is the substance of §5's
  argument for shipping unbound.
- **Windows behaviour generally.** Every Windows statement in §3 is from
  Microsoft's documentation and from this repo's own source. Nothing was run on
  Windows.
