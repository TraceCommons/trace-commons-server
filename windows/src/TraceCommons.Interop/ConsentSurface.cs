using System;
using System.Text.Json;

namespace TraceCommons.Interop;

/// <summary>
/// The consent surface's wording, across the C ABI.
///
/// <para>
/// Nothing in this file is a word. The sentences cross as JSON, already
/// assembled, and the choice between the two tooltips crosses as its own
/// call, so this shell fills in no template and takes no branch of its own.
/// </para>
/// </summary>
public static class ConsentSurface
{
    /// <summary>
    /// Every fixed sentence on the surface, or null when the call failed or
    /// the payload will not parse.
    ///
    /// Null, never a partly-filled record: a blank where a safety claim goes
    /// is worse than nothing, and a C#-authored claim is worse than both.
    /// The caller decides what to show when the words are not available.
    /// </summary>
    public static ConsentCopy? Copy() =>
        Parse(NativeMethods.TakeOwnedString(NativeMethods.tc_consent_copy()));

    /// <summary>
    /// The payload half of <see cref="Copy"/>, split out so it is testable
    /// without the cdylib. The native call is a one-liner; this is where the
    /// behaviour that can actually be wrong lives.
    /// </summary>
    internal static ConsentCopy? Parse(string? json)
    {
        if (string.IsNullOrWhiteSpace(json))
        {
            return null;
        }

        try
        {
            ConsentCopy? copy = JsonSerializer.Deserialize<ConsentCopy>(json);
            if (copy is null)
            {
                return null;
            }

            foreach (string sentence in copy.Sentences)
            {
                if (string.IsNullOrEmpty(sentence))
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
    /// The tooltip that explains the current answer, chosen by the ABI.
    ///
    /// Null only on a caught panic. Do not recover this by picking between
    /// <see cref="ConsentCopy.ReadyHelp"/> and
    /// <see cref="ConsentCopy.NotPinnedHelp"/> here: the branch crosses so
    /// that three shells cannot each keep their own copy of it.
    /// </summary>
    public static string? GateHelp(bool pinned) =>
        NativeMethods.TakeOwnedString(NativeMethods.tc_consent_gate_help(pinned ? 1 : 0));
}
