using System;
using System.Collections.Generic;
using System.Text.Json;

namespace TraceCommons.Interop;

/// <summary>
/// How firmly one <c>private_inference_state</c> reads.
///
/// Five values. The ABI numbering these decode from is deliberately disjoint
/// from <see cref="RoutingTone"/>'s and the witness surface's: do not share a
/// mapper with either, because a routing mapper would answer neutral for
/// every value here and turn a refusal into "nothing to say".
/// </summary>
public enum PrivateInferenceTone
{
    /// <summary>Nothing is running and nothing is claimed.</summary>
    Neutral,

    /// <summary>On, and this app is not the one answering.</summary>
    Held,

    /// <summary>
    /// On, answering, and with somewhere to pass calls on to. The only value
    /// that may be painted as working.
    /// </summary>
    Clear,

    /// <summary>
    /// On, and something on this machine wants attention before a call can
    /// get through. This is <c>running_no_backends</c>.
    /// </summary>
    Attention,

    /// <summary>
    /// Asked for and not happening. Always paired with a sentence naming the
    /// way out.
    /// </summary>
    Refused,
}

/// <summary>
/// What the daemon reported about the listener.
///
/// The label is carried as the daemon's own string and handed to the shared
/// table. It is deliberately not parsed into an enum here: a state a later
/// daemon grows would then have to be spelled in this shell before it could
/// be shown, and the shared table already answers an unknown label safely.
/// </summary>
public readonly record struct PrivateInferenceState(string Label, ushort? Port)
{
    /// <summary>
    /// A daemon that has never heard of the field reports nothing, which is
    /// the empty label -- answered by the shared table with the sentence that
    /// claims nothing. Never null: a card with a switch and no line beneath
    /// it is the shape that says "on" over a listener that refused to start.
    /// </summary>
    public static PrivateInferenceState From(PrivateInferenceStateSnapshot? snapshot) =>
        new(snapshot?.State ?? string.Empty, snapshot?.Port);
}

/// <summary>
/// The private-inference surface, across the C ABI.
///
/// Holds no words and owns no branch. Every sentence and both decisions come
/// from <c>crates/trace-commons-contributor/src/private_inference_copy.rs</c>.
/// </summary>
public static class PrivateInferenceSurface
{
    /// <summary>The <c>set_settings</c> key for the switch.</summary>
    public const string SettingsKey = "private_inference";

    /// <summary>
    /// The <c>set_settings</c> key recording that the question was put.
    /// </summary>
    public const string OfferSeenKey = "private_inference_offer_seen";

    /// <summary>
    /// Every fixed word, or null if the export or the decode failed. Both
    /// surfaces render nothing at all in that case: an offer missing its
    /// exposure paragraph is worse than no offer.
    /// </summary>
    public static PrivateInferenceCopy? Copy() =>
        Parse(NativeMethods.TakeOwnedString(NativeMethods.tc_private_inference_copy()));

    /// <summary>
    /// All or nothing. Split out from <see cref="Copy"/> so the refusal rule
    /// is testable without the cdylib.
    /// </summary>
    internal static PrivateInferenceCopy? Parse(string? json)
    {
        if (string.IsNullOrWhiteSpace(json))
        {
            return null;
        }

        try
        {
            PrivateInferenceCopy? copy = JsonSerializer.Deserialize<PrivateInferenceCopy>(json);
            if (copy is null)
            {
                return null;
            }

            foreach (string sentence in copy.Sentences)
            {
                if (string.IsNullOrWhiteSpace(sentence))
                {
                    return null;
                }
            }

            return copy;
        }
        catch (JsonException)
        {
            return null;
        }
    }

    /// <summary>
    /// The sentence for one state. Falls back to the payload's own off
    /// sentence when the Rust caught a panic -- the one that claims nothing,
    /// never the one that says it is running.
    /// </summary>
    public static string StateLine(PrivateInferenceState state, PrivateInferenceCopy copy)
    {
        ArgumentNullException.ThrowIfNull(copy);
        return NativeMethods.TakeOwnedString(
                NativeMethods.tc_private_inference_state_line(state.Label))
            ?? copy.StateOff;
    }

    /// <summary>The tone that sentence is painted in.</summary>
    public static PrivateInferenceTone Tone(PrivateInferenceState state) =>
        FromAbiTone(NativeMethods.tc_private_inference_state_tone(state.Label));

    /// <summary>
    /// Where it is answering, or the empty string when there is no port. An
    /// empty string is drawn as no line rather than as a blank one.
    /// </summary>
    public static string ServingLine(PrivateInferenceState state) =>
        NativeMethods.TakeOwnedString(
            NativeMethods.tc_private_inference_serving_line(state.Port ?? 0))
        ?? string.Empty;

    /// <summary>
    /// Whether to put the offer in front of the contributor. Asked of the
    /// shared table, never decided here.
    /// </summary>
    public static bool ShouldOffer(bool answered, bool on) =>
        NativeMethods.tc_private_inference_should_offer(answered ? 1 : 0, on ? 1 : 0) != 0;

    /// <summary>
    /// The <c>set_settings</c> body for one answer to the offer.
    ///
    /// Declining writes the marker ALONE. It must never write the switch, not
    /// even as false: the switch is already false, and writing it would make
    /// a refusal indistinguishable from a change.
    ///
    /// Accepting writes both in one call, so an accept cannot record the
    /// answer and fail to start, or start and fail to record.
    /// </summary>
    public static string SerializeOfferAnswer(bool accepted)
    {
        var declarations = new Dictionary<string, bool> { [OfferSeenKey] = true };
        if (accepted)
        {
            declarations[SettingsKey] = true;
        }

        return JsonSerializer.Serialize(declarations);
    }

    /// <summary>
    /// The <c>set_settings</c> body for the switch on the settings card.
    ///
    /// Carries the marker too: a contributor who found the switch on their
    /// own has answered the question and must not be asked it later.
    /// </summary>
    public static string SerializeSwitch(bool on) =>
        JsonSerializer.Serialize(
            new Dictionary<string, bool> { [SettingsKey] = on, [OfferSeenKey] = true });

    /// <summary>
    /// The extra sentence the quit confirmation carries while the switch is
    /// on, or null. Null when it is off: a contributor who never turned it on
    /// should not be warned about losing it.
    /// </summary>
    public static string? QuitDetail(bool on, PrivateInferenceCopy? copy) =>
        on && copy is not null ? copy.QuitAlsoStops : null;

    /// <summary>
    /// The ABI value, spelled out rather than cast.
    ///
    /// Anything unknown is <see cref="PrivateInferenceTone.Neutral"/>: the
    /// dangerous value on this surface is <c>Clear</c>, so a state a later
    /// daemon grows must claim nothing rather than be drawn as running.
    /// </summary>
    internal static PrivateInferenceTone FromAbiTone(int value) =>
        value switch
        {
            AbiToneHeld => PrivateInferenceTone.Held,
            AbiToneClear => PrivateInferenceTone.Clear,
            AbiToneAttention => PrivateInferenceTone.Attention,
            AbiToneRefused => PrivateInferenceTone.Refused,
            _ => PrivateInferenceTone.Neutral,
        };

    private const int AbiToneHeld = 21;
    private const int AbiToneClear = 22;
    private const int AbiToneAttention = 23;
    private const int AbiToneRefused = 24;
}
