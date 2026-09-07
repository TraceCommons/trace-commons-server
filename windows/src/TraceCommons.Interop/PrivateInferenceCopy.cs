using System.Text.Json.Serialization;

namespace TraceCommons.Interop;

/// <summary>
/// Every fixed sentence on the private-inference offer and settings card, as
/// <c>tc_private_inference_copy</c> exports it.
///
/// A pure carrier. Nothing here is authored in this shell: the three shells
/// print one offer, and the paragraph most at risk of being paraphrased is
/// the one saying what turning the switch on exposes.
///
/// Every property defaults to the empty string so a malformed payload cannot
/// throw during deserialisation; <see cref="PrivateInferenceSurface.Parse"/>
/// then refuses the whole object rather than handing a screen a blank where a
/// sentence should be.
/// </summary>
public sealed record PrivateInferenceCopy
{
    [JsonPropertyName("offer_title")]
    public string OfferTitle { get; init; } = string.Empty;

    [JsonPropertyName("offer_what")]
    public string OfferWhat { get; init; } = string.Empty;

    /// <summary>
    /// What turning the switch on exposes. The one sentence this surface will
    /// not render without.
    /// </summary>
    [JsonPropertyName("offer_exposure")]
    public string OfferExposure { get; init; } = string.Empty;

    [JsonPropertyName("offer_no_repoint")]
    public string OfferNoRepoint { get; init; } = string.Empty;

    [JsonPropertyName("offer_accept")]
    public string OfferAccept { get; init; } = string.Empty;

    [JsonPropertyName("offer_decline")]
    public string OfferDecline { get; init; } = string.Empty;

    [JsonPropertyName("offer_asked_once")]
    public string OfferAskedOnce { get; init; } = string.Empty;

    /// <summary>
    /// The rail label for the top-level destination this surface owns.
    ///
    /// Read rather than typed. A shell that spelled the label itself would
    /// keep spelling the old one after a rename in the Rust, and this is the
    /// one word in the whole surface a contributor navigates by.
    /// </summary>
    [JsonPropertyName("destination")]
    public string Destination { get; init; } = string.Empty;

    /// <summary>The one line under the destination's title saying what it is for.</summary>
    [JsonPropertyName("subtitle")]
    public string Subtitle { get; init; } = string.Empty;

    [JsonPropertyName("settings_title")]
    public string SettingsTitle { get; init; } = string.Empty;

    [JsonPropertyName("settings_toggle")]
    public string SettingsToggle { get; init; } = string.Empty;

    [JsonPropertyName("settings_applies_at_once")]
    public string SettingsAppliesAtOnce { get; init; } = string.Empty;

    [JsonPropertyName("state_unreported")]
    public string StateUnreported { get; init; } = string.Empty;

    [JsonPropertyName("state_unknown")]
    public string StateUnknown { get; init; } = string.Empty;

    [JsonPropertyName("state_stopping")]
    public string StateStopping { get; init; } = string.Empty;

    [JsonPropertyName("state_off")]
    public string StateOff { get; init; } = string.Empty;

    [JsonPropertyName("state_running")]
    public string StateRunning { get; init; } = string.Empty;

    [JsonPropertyName("state_running_no_backends")]
    public string StateRunningNoBackends { get; init; } = string.Empty;

    [JsonPropertyName("state_running_elsewhere")]
    public string StateRunningElsewhere { get; init; } = string.Empty;

    [JsonPropertyName("state_port_in_use")]
    public string StatePortInUse { get; init; } = string.Empty;

    [JsonPropertyName("state_start_failed")]
    public string StateStartFailed { get; init; } = string.Empty;

    [JsonPropertyName("state_crashed")]
    public string StateCrashed { get; init; } = string.Empty;

    /// <summary>
    /// The extra line the quit confirmation carries while the switch is on.
    /// The rest of that dialog is authored in this shell; this sentence
    /// deliberately is not.
    /// </summary>
    [JsonPropertyName("quit_also_stops")]
    public string QuitAlsoStops { get; init; } = string.Empty;

    /// <summary>A write could not be confirmed; persistence may still have happened.</summary>
    [JsonPropertyName("write_unconfirmed")]
    public string WriteUnconfirmed { get; init; } = string.Empty;

    /// <summary>The sentence the settings card shows once the control has moved out of it.</summary>
    [JsonPropertyName("settings_moved")]
    public string SettingsMoved { get; init; } = string.Empty;

    /// <summary>The tray action while it is on. Turning it off needs no sentence in front of it.</summary>
    [JsonPropertyName("tray_turn_off")]
    public string TrayTurnOff { get; init; } = string.Empty;

    /// <summary>
    /// The tray action while it is off. Opens the screen rather than acting: turning it ON
    /// changes what anything else on this computer may send through, which is not a decision
    /// to take from a menu with the consequence off-screen.
    /// </summary>
    [JsonPropertyName("tray_open_to_turn_on")]
    public string TrayOpenToTurnOn { get; init; } = string.Empty;

    /// <summary>Every sentence for the complete-payload check, not a rendering order.</summary>
    public string[] Sentences =>
        new[]
        {
            OfferTitle,
            OfferWhat,
            OfferExposure,
            OfferNoRepoint,
            OfferAccept,
            OfferDecline,
            OfferAskedOnce,
            Destination,
            Subtitle,
            SettingsTitle,
            SettingsToggle,
            SettingsAppliesAtOnce,
            StateUnreported,
            StateUnknown,
            StateStopping,
            StateOff,
            StateRunning,
            StateRunningNoBackends,
            StateRunningElsewhere,
            StatePortInUse,
            StateStartFailed,
            StateCrashed,
            QuitAlsoStops,
            WriteUnconfirmed,
            SettingsMoved,
            TrayTurnOff,
            TrayOpenToTurnOn,
        };
}
