//! The model-calls screen: whether this computer answers the model calls
//! made on it, and what the listener is actually doing.
//!
//! Its own destination rather than a card five sections down Settings. The
//! switch here is the one that makes this machine answer calls for whatever
//! else is running on it, and a contributor who has turned it on is owed a
//! place they can look at to see whether it is on right now.
//!
//! **Every sentence is read from `copy`**, which re-exports
//! `trace_commons_contributor::private_inference_copy` -- the same
//! definition the macOS and Windows shells reach across the C ABI. Nothing
//! here is authored in this view. The one word this surface may never say is
//! that the calls are private: turning this on does not make a call private,
//! it moves where the call is answered, and each call still goes on to
//! whoever was configured to answer it. The shared module carries the sweep
//! that enforces that; this view stays out of its way by never retyping a
//! sentence.
//!
//! **The indicator follows the tone, never the switch.** The switch shows
//! what was asked for; the tone -- and only `Clear` in it -- shows what is
//! true. A listener that refused to start leaves the switch on, and a screen
//! that painted a green dot from the boolean would be reporting a service
//! running that is not.
//!
//! **This window carries the whole capability.** GNOME has no tray (see
//! `ui/mod.rs`), so the tray entry beside this is only ever a shortcut in:
//! nothing on this screen may be reachable only from there.

use std::rc::Rc;

use adw::prelude::*;

use super::style::{self, Tone, space};
use super::{App, COLUMN_MAX, COLUMN_TIGHTEN};
use crate::copy;

/// The screen's widgets. Built once and refilled, like every other screen
/// in this window: a refresh runs on every daemon event, and rebuilding the
/// switch under a contributor's finger would drop the press.
pub struct PrivateInferenceView {
    pub root: gtk::Box,
    /// What was asked for. Insensitive until the daemon's own answer has
    /// arrived, so a press cannot write a value nothing confirmed.
    switch: gtk::Switch,
    /// What the listener is actually doing: one toned sentence, and the
    /// serving line under it when there is a port. Rebuilt on each render.
    status: gtk::Box,
    /// Set while a render is writing the daemon's answer into the switch,
    /// so the signal that fires is not mistaken for a contributor acting.
    filling: std::cell::Cell<bool>,
    /// The last confirmed requested setting. `None` is "not yet heard from",
    /// which is not an off declaration.
    confirmed: std::cell::Cell<Option<bool>>,
    write_pending: std::cell::Cell<bool>,
}

impl Default for PrivateInferenceView {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivateInferenceView {
    pub fn new() -> Self {
        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(space::XL)
            .margin_top(space::L)
            .margin_bottom(space::XXL)
            .margin_start(space::XL)
            .margin_end(space::XL)
            .build();

        content.append(&style::section(copy::PRIVATE_INFERENCE_TITLE));
        style::append_body(&content, copy::PRIVATE_INFERENCE_SUBTITLE);

        let card = style::card(gtk::Orientation::Vertical, space::M);
        style::append_body(&card, copy::PRIVATE_INFERENCE_OFFER_WHAT);
        // The exposure sentence is on this screen, not only in the offer. A
        // contributor who declined and came back months later is making the
        // same decision and is owed the same sentence.
        style::append_body(&card, copy::PRIVATE_INFERENCE_OFFER_EXPOSURE);

        let row = gtk::Box::new(gtk::Orientation::Horizontal, space::S);
        let label = gtk::Label::builder()
            .label(copy::PRIVATE_INFERENCE_TOGGLE)
            .xalign(0.0)
            .wrap(true)
            .hexpand(true)
            .build();
        label.add_css_class("tc-body");
        let switch = gtk::Switch::builder().halign(gtk::Align::End).build();
        switch.set_sensitive(false);
        switch.set_valign(gtk::Align::Center);
        switch.update_property(&[gtk::accessible::Property::Label(
            copy::PRIVATE_INFERENCE_TOGGLE,
        )]);
        row.append(&label);
        row.append(&switch);
        card.append(&row);

        // Drawn before anything has been asked, so the screen never opens
        // on a blank space where the state will be. Unreported is what is
        // true at that moment, and the shared table has a sentence for it.
        let status = gtk::Box::new(gtk::Orientation::Vertical, space::XS);
        status.append(&tone_row(
            copy::PRIVATE_INFERENCE_STATE_UNKNOWN,
            Tone::Neutral,
        ));
        card.append(&status);
        style::append_caveat(&card, copy::PRIVATE_INFERENCE_APPLIES_AT_ONCE);
        content.append(&card);

        let clamp = adw::Clamp::builder()
            .maximum_size(COLUMN_MAX)
            .tightening_threshold(COLUMN_TIGHTEN)
            .child(&content)
            .build();
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .child(&clamp)
            .build();
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.add_css_class("tc-root");
        root.append(&scroller);

        Self {
            root,
            switch,
            status,
            filling: std::cell::Cell::new(false),
            confirmed: std::cell::Cell::new(None),
            write_pending: std::cell::Cell::new(false),
        }
    }
}

/// A glyph and a sentence, read as one statement.
///
/// A local copy of the shape Settings uses rather than a shared helper: the
/// two screens are free to diverge, and the three lines here are not worth
/// coupling them over.
fn tone_row(label: &str, tone: Tone) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, space::S);
    let glyph = gtk::Label::new(Some(tone.glyph()));
    glyph.add_css_class(tone.css());
    glyph.set_valign(gtk::Align::Start);
    row.append(&glyph);
    let text = gtk::Label::builder()
        .label(label)
        .xalign(0.0)
        .wrap(true)
        .hexpand(true)
        .build();
    text.add_css_class("tc-body");
    row.append(&text);
    row.update_property(&[gtk::accessible::Property::Label(label)]);
    row
}

/// The shared tone onto this shell's palette.
///
/// NOT A BRANCH TABLE ON THE STATE: which tone each state reads in is
/// decided once, in `private_inference_copy`, beside the sentence it goes
/// with. This only carries that answer onto `style::Tone`.
///
/// **No catch-all arm, deliberately.** A tone this shell has not been taught
/// must fail to compile rather than fall through to something that reads as
/// working. `indicator_reads_as_working` pins the other half of that rule:
/// `Tone::Clear` comes out of here for `Clear` and for nothing else.
fn indicator_tone(tone: copy::PrivateInferenceTone) -> Tone {
    match tone {
        copy::PrivateInferenceTone::Neutral => Tone::Neutral,
        copy::PrivateInferenceTone::Held => Tone::Held,
        copy::PrivateInferenceTone::Clear => Tone::Clear,
        copy::PrivateInferenceTone::Attention => Tone::Attention,
        copy::PrivateInferenceTone::Refused => Tone::Refused,
    }
}

pub fn wire(app: &Rc<App>) {
    // The switch writes on its own: flipping it IS the contributor acting,
    // and there is nothing else on the screen to fill in first.
    let a = Rc::clone(app);
    app.private_inference
        .switch
        .connect_active_notify(move |sw| {
            if a.private_inference.filling.get() {
                return;
            }
            send(&a, sw.is_active());
        });
}

/// Ask the daemon what it is doing, and draw that.
///
/// Read on every refresh rather than latched in this window: the listener
/// can stop without this process doing anything, and a latched answer would
/// keep saying it was running.
pub fn refresh(app: &Rc<App>) {
    app.call("get_settings", serde_json::json!({}), |app, result| {
        let Ok(Ok(settings)) = result.map(serde_json::from_value::<crate::model::Settings>) else {
            return;
        };
        render(app, &settings);
    });
}

/// The switch, and what actually happened underneath it.
///
/// The switch shows what was ASKED FOR and the row beneath shows what
/// happened, because they differ exactly when it matters: a listener that
/// refused to start leaves the switch on, and a screen with only the switch
/// on it would say the thing is running.
pub fn render(app: &Rc<App>, settings: &crate::model::Settings) {
    let view = &app.private_inference;
    view.confirmed.set(Some(settings.private_inference));
    // The tray's action row points off the confirmed switch, so it is
    // written here, where the daemon's own answer arrives, rather than
    // optimistically at the press. It is the switch's position and nothing
    // that paints reads it -- see `App::tray`.
    app.tray.set_answering(settings.private_inference);
    view.switch.set_sensitive(!view.write_pending.get());
    view.filling.set(true);
    view.switch.set_active(settings.private_inference);
    view.filling.set(false);

    let status = &view.status;
    while let Some(child) = status.first_child() {
        status.remove(&child);
    }
    // A daemon that has never heard of the field sends nothing, and the
    // shared table renders that as unreported, separately from an unfamiliar
    // nonempty state. Neither is evidence that a listener stopped.
    let (label, port) = match settings.private_inference_state.as_ref() {
        Some(state) => (state.state.as_str(), state.port),
        None => ("", None),
    };
    status.append(&tone_row(
        copy::private_inference_state_line(label),
        indicator_tone(copy::private_inference_state_tone(label)),
    ));
    // Assembled on the shared side, and empty when there is no port. An
    // empty sentence is drawn as no row at all rather than as a blank line.
    let serving = copy::private_inference_serving_line(port);
    if !serving.is_empty() {
        style::append_meta(status, serving);
    }
}

/// Stop answering model calls, for the tray's one write.
///
/// `false` and not a flip. The tray may reduce what this computer answers
/// and may not enlarge it -- `tray::TrayRequest` has no word for the other
/// direction -- and a flip here would supply one whenever the menu that was
/// pressed had gone stale.
///
/// It goes through [`send`] rather than around it so the one write path
/// keeps its guards: the confirmed-value check, the in-flight check, and the
/// rule that the screen is drawn from the daemon's echo and never from what
/// was asked for.
pub(crate) fn turn_off(app: &Rc<App>) {
    send(app, false);
}

/// Write the switch, and render from the daemon's echo.
///
/// Never optimistic: what comes back carries `private_inference_state`, and
/// that is the only thing that knows whether the listener actually started.
/// Turning it on from here also records that the question has been answered,
/// so the offer does not appear on the next launch for a contributor who
/// found the screen themselves.
fn send(app: &Rc<App>, on: bool) {
    let view = &app.private_inference;
    // The signal fires after the thumb moved. Put back the confirmed value
    // while waiting, so an error cannot leave an unconfirmed request on
    // screen.
    view.filling.set(true);
    view.switch
        .set_active(view.confirmed.get().unwrap_or(false));
    view.filling.set(false);
    if view.confirmed.get().is_none() || view.write_pending.replace(true) {
        return;
    }
    view.switch.set_sensitive(false);
    app.call(
        "set_settings",
        serde_json::json!({
            "private_inference": on,
            "private_inference_offer_seen": true,
        }),
        move |app, result| {
            app.private_inference.write_pending.set(false);
            app.private_inference.switch.set_sensitive(true);
            let confirmed = result.as_ref().is_ok_and(|value| {
                copy::private_inference_write_confirmed(
                    Some(on),
                    value
                        .get("private_inference_offer_seen")
                        .and_then(serde_json::Value::as_bool),
                    value
                        .get("private_inference")
                        .and_then(serde_json::Value::as_bool),
                )
            });
            let settings = result
                .ok()
                .and_then(|value| serde_json::from_value::<crate::model::Settings>(value).ok());
            if let Some(settings) = settings.filter(|_| confirmed) {
                render(app, &settings);
            } else {
                app.toast(copy::PRIVATE_INFERENCE_WRITE_UNCONFIRMED);
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("private_inference.rs");

    /// The rule the whole screen hangs on: `Clear` is the only tone that may
    /// be painted as working, and it is the only tone that reaches the
    /// palette's `Clear`. Held, Attention and Refused each keep their own
    /// distinct glyph and colour, so none of them can be read as "on".
    #[test]
    fn only_a_clear_tone_is_painted_as_working() {
        for tone in [
            copy::PrivateInferenceTone::Neutral,
            copy::PrivateInferenceTone::Held,
            copy::PrivateInferenceTone::Clear,
            copy::PrivateInferenceTone::Attention,
            copy::PrivateInferenceTone::Refused,
        ] {
            assert_eq!(
                indicator_tone(tone) == Tone::Clear,
                tone.reads_as_working(),
                "{tone:?} is painted as working exactly when it reads as working"
            );
        }
    }

    /// Every non-working tone must also be distinguishable from every other
    /// one -- a screen that collapsed Refused into Held would be answering a
    /// different question than the daemon was asked.
    #[test]
    fn each_tone_keeps_its_own_glyph() {
        let mut seen: Vec<&str> = Vec::new();
        for tone in [
            copy::PrivateInferenceTone::Neutral,
            copy::PrivateInferenceTone::Held,
            copy::PrivateInferenceTone::Clear,
            copy::PrivateInferenceTone::Attention,
            copy::PrivateInferenceTone::Refused,
        ] {
            let glyph = indicator_tone(tone).glyph();
            assert!(
                !seen.contains(&glyph),
                "{tone:?} reuses the glyph {glyph:?}"
            );
            seen.push(glyph);
        }
    }

    /// The state a daemon that has never heard of the field reports, and any
    /// label this build does not know, must not read as working either.
    #[test]
    fn an_unreported_or_unknown_state_is_not_painted_as_working() {
        for label in ["", "unknown_state", "stopping", "port_in_use", "crashed"] {
            let tone = copy::private_inference_state_tone(label);
            assert_ne!(
                indicator_tone(tone),
                Tone::Clear,
                "{label:?} must not be painted as working"
            );
        }
    }

    /// The indicator reads the tone. Nothing in the render path may reach
    /// the requested boolean to decide how the state row is painted -- the
    /// switch says what was asked for, the row says what is true.
    #[test]
    fn the_indicator_is_not_drawn_from_the_requested_setting() {
        let body = SOURCE
            .split("pub fn render(")
            .nth(1)
            .expect("render is in this file")
            .split("\nfn send(")
            .next()
            .expect("render ends before send");
        assert!(body.contains("copy::private_inference_state_line(label)"));
        assert!(body.contains("copy::private_inference_state_tone(label)"));
        // `settings.private_inference` may reach the switch and nothing
        // else; the tone row is built from the daemon's reported state.
        let after_switch = body
            .split("view.filling.set(false);")
            .nth(1)
            .expect("the switch is filled before the status row is built");
        assert!(
            !after_switch.contains("settings.private_inference,"),
            "the status row must not branch on the requested setting"
        );
    }

    /// The one write anything outside this module can ask for turns model
    /// calls OFF, and there is no way to ask for the other direction.
    ///
    /// `send` is private, so `turn_off` is the whole of this screen's write
    /// surface for the rest of the application -- the tray included. That is
    /// the structural half; this is the other half, that `turn_off` writes a
    /// fixed `false` rather than flipping whatever it finds. A flip would
    /// turn a stale tray menu into an enable nobody asked for, and enabling
    /// is the direction that must happen with the sentence about what it
    /// exposes on screen.
    #[test]
    fn the_only_write_reachable_from_outside_stops_answering() {
        let body = SOURCE
            .split("pub(crate) fn turn_off(app: &Rc<App>) {")
            .nth(1)
            .expect("turn_off is in this file")
            .split("\n}")
            .next()
            .expect("turn_off closes");
        assert!(
            body.contains("send(app, false)"),
            "turn_off stopped writing false"
        );
        assert!(
            !body.contains("true"),
            "turn_off can reach the on direction"
        );
        // The module's own code, without the tests that quote it.
        let code = SOURCE
            .split("\n#[cfg(test)]")
            .next()
            .expect("the module has a body before its tests");
        assert_eq!(
            code.matches("pub(crate) fn ").count(),
            1,
            "a second write escaped this module"
        );
        assert!(
            !code.contains("pub fn send("),
            "the write path is reachable without going through turn_off"
        );
    }

    /// Nothing on this screen is authored here. Every string literal in the
    /// constructor is a CSS class; the words come from `copy`.
    #[test]
    fn the_screen_authors_no_sentence_of_its_own() {
        let body = SOURCE
            .split("    pub fn new() -> Self {")
            .nth(1)
            .expect("the constructor is in this file")
            .split("\n        Self {")
            .next()
            .expect("the constructor ends by returning Self");
        for literal in body.split('"').skip(1).step_by(2) {
            assert!(
                literal.starts_with("tc-"),
                "{literal:?} is a sentence written in this view rather than read from copy"
            );
        }
    }

    /// The write records that the question has been answered, so the
    /// first-run offer is not asked again of somebody who found this screen.
    #[test]
    fn turning_it_on_here_also_answers_the_offer() {
        let body = SOURCE
            .split("fn send(app: &Rc<App>, on: bool)")
            .nth(1)
            .expect("send is in this file");
        assert!(body.contains("\"private_inference\": on"));
        assert!(body.contains("\"private_inference_offer_seen\": true"));
    }
}
