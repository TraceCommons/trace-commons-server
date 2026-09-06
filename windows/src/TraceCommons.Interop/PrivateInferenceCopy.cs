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

    [JsonPropertyName("settings_title")]
    public string SettingsTitle { get; init; } = string.Empty;

    [JsonPropertyName("settings_toggle")]
    public string SettingsToggle { get; init; } = string.Empty;

    [JsonPropertyName("settings_applies_at_once")]
    public string SettingsAppliesAtOnce { get; init; } = string.Empty;

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

    /// <summary>
    /// Every sentence, for the "nothing arrived blank" check. Not a rendering
    /// order.
    /// </summary>
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
            SettingsTitle,
            SettingsToggle,
            SettingsAppliesAtOnce,
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
        };
}
