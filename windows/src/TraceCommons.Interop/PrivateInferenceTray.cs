namespace TraceCommons.Interop;

/// <summary>
/// The model-calls entry in the notification-area menu: what it says, and
/// whether anything on it may be drawn as working.
///
/// <para>
/// Computed here rather than in the tray class so it can be tested off
/// Windows, and so the one rule that matters is written once. That rule is
/// that <see cref="ReadsAsWorking"/> comes from the daemon's reported state
/// and never from <see cref="On"/>. The switch is what was asked for; the
/// state is what happened. A contributor who turned it on over a port that
/// was already taken has <c>On == true</c> and a listener that never
/// started, and a tray that lit a glyph from the switch would tell them
/// their model calls are being answered here when nothing is answering.
/// </para>
/// <para>
/// <see cref="On"/> is still carried, because the menu holds the switch
/// itself and a check mark on a switch is that switch's own position rather
/// than a claim about the world. Nothing else may read it.
/// </para>
/// </summary>
public readonly record struct PrivateInferenceTrayEntry
{
    /// <summary>
    /// Whether the entry belongs in the menu at all. False when the words did
    /// not arrive: a menu row carrying a switch and no sentence beside it is
    /// the shape that says "on" over a listener that refused to start, and
    /// the settings card refuses to render for the same reason.
    /// </summary>
    public bool Available { get; init; }

    /// <summary>The destination's own name, as the Rust spells it.</summary>
    public string Label { get; init; }

    /// <summary>The sentence for the reported state.</summary>
    public string StateText { get; init; }

    /// <summary>The switch's label.</summary>
    public string ToggleText { get; init; }

    /// <summary>Where the switch is. The menu's check mark, and nothing else.</summary>
    public bool On { get; init; }

    /// <summary>
    /// Whether anything on this entry may be drawn as working. Derived from
    /// the reported state alone.
    /// </summary>
    public bool ReadsAsWorking { get; init; }

    /// <summary>
    /// The entry for one payload, one reported state and one switch position.
    /// </summary>
    public static PrivateInferenceTrayEntry For(
        PrivateInferenceCopy? copy, PrivateInferenceState state, bool on)
    {
        if (copy is null)
        {
            return new PrivateInferenceTrayEntry
            {
                Available = false,
                Label = string.Empty,
                StateText = string.Empty,
                ToggleText = string.Empty,
                On = on,
                ReadsAsWorking = false,
            };
        }

        return new PrivateInferenceTrayEntry
        {
            Available = true,
            Label = copy.Destination,
            StateText = PrivateInferenceSurface.StateLine(state, copy),
            ToggleText = copy.SettingsToggle,
            On = on,

            // The state, never `on`. See the class remarks.
            ReadsAsWorking = PrivateInferenceSurface.Tone(state).ReadsAsWorking(),
        };
    }
}
