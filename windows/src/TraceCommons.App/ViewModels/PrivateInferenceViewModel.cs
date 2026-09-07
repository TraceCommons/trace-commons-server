using System;
using System.ComponentModel;
using System.Threading.Tasks;
using TraceCommons.Interop;

namespace TraceCommons.App.ViewModels;

/// <summary>
/// The model-calls destination: everything its page shows, and the one write
/// it can make.
///
/// <para>
/// Not one sentence is composed here. Every word arrives from
/// <c>crates/trace-commons-contributor/src/private_inference_copy.rs</c>
/// across the C ABI, including the destination's own name -- the rail label
/// is the one word a contributor navigates by, and a shell that spelled it
/// itself would go on spelling the old one after a rename in the Rust.
/// </para>
/// <para>
/// The whole page is hidden when those words did not arrive, rather than
/// drawn with a switch and nothing beside it. That shape is what says "on"
/// over a listener that refused to start.
/// </para>
/// <para>
/// <b>The status is read from the tone, never from the switch.</b> The switch
/// reports what was asked for; the tone reports what the daemon says
/// happened. They disagree exactly when it matters -- the switch on, the port
/// already taken -- and every indicator this shell draws for this surface,
/// here and on the rail and in the tray, comes off the tone.
/// </para>
/// </summary>
public sealed class PrivateInferenceViewModel : INotifyPropertyChanged
{
    private readonly DaemonHost _host;

    /// <summary>
    /// Every fixed word, read once across the ABI. Null when the export or
    /// the decode failed, and then the page renders nothing at all.
    /// </summary>
    private readonly PrivateInferenceCopy? _copy = PrivateInferenceSurface.Copy();

    private PrivateInferenceState _state = new(string.Empty, null);
    private bool _on;
    private bool _busy;
    private bool _loaded;

    public PrivateInferenceViewModel(DaemonHost host)
    {
        _host = host ?? throw new ArgumentNullException(nameof(host));
    }

    public event PropertyChangedEventHandler? PropertyChanged;

    /// <summary>Whether the words arrived at all.</summary>
    public bool Available => _copy is not null;

    public bool ControlsEnabled => !_busy && _copy is not null;

    public string Title => _copy?.SettingsTitle ?? string.Empty;

    public string Subtitle => _copy?.Subtitle ?? string.Empty;

    public string What => _copy?.OfferWhat ?? string.Empty;

    /// <summary>
    /// What turning it on exposes. On this page as well as in the first-run
    /// offer: a contributor who declined and came back later is making the
    /// same decision and is owed the same sentence.
    /// </summary>
    public string Exposure => _copy?.OfferExposure ?? string.Empty;

    public string ToggleText => _copy?.SettingsToggle ?? string.Empty;

    public string AppliesAtOnce => _copy?.SettingsAppliesAtOnce ?? string.Empty;

    /// <summary>Where the switch is. The switch's own position, and nothing else.</summary>
    public bool Enabled => _on;

    /// <summary>The sentence for whatever the daemon last reported.</summary>
    public string StateText => _copy is null
        ? string.Empty
        : PrivateInferenceSurface.StateLine(_state, _copy);

    /// <summary>The reported port, or the empty string when there is none.</summary>
    public string ServingText => PrivateInferenceSurface.ServingLine(_state);

    public bool HasServingText => ServingText.Length > 0;

    private PrivateInferenceTone StateTone => PrivateInferenceSurface.Tone(_state);

    /// <summary>
    /// Whether this page may paint anything as working.
    ///
    /// The tone, and only the tone. Reading <see cref="Enabled"/> here is the
    /// fail-open this surface exists to prevent.
    /// </summary>
    public bool IsWorking => StateTone.ReadsAsWorking();

    // One flag per tone rather than a converted foreground: three of the
    // sentences begin with the same two words, so a colour picked by reading
    // the sentence back would be picked by matching a prefix.

    public bool StateIsNeutral => StateTone == PrivateInferenceTone.Neutral;

    public bool StateIsHeld => StateTone == PrivateInferenceTone.Held;

    public bool StateIsClear => StateTone == PrivateInferenceTone.Clear;

    public bool StateIsAttention => StateTone == PrivateInferenceTone.Attention;

    public bool StateIsRefused => StateTone == PrivateInferenceTone.Refused;

    /// <summary>
    /// The sentence shown when a write could not be confirmed, or the empty
    /// string. The payload's, like everything else here.
    /// </summary>
    public string Notice
    {
        get => _notice;
        private set
        {
            if (_notice == value)
            {
                return;
            }

            _notice = value;
            Raise(nameof(Notice));
            Raise(nameof(HasNotice));
        }
    }

    private string _notice = string.Empty;

    public bool HasNotice => _notice.Length > 0;

    /// <summary>
    /// Reads the switch and the reported state once the page is on screen.
    /// </summary>
    public async Task LoadAsync()
    {
        _busy = true;
        Raise(nameof(ControlsEnabled));
        try
        {
            DaemonResponse response = await _host
                .CallAsync(DaemonProtocol.Methods.GetSettings)
                .ConfigureAwait(true);
            Fill(response.ResultAs<DaemonSettingsSnapshot>());
            _loaded = true;
        }
        catch
        {
            System.Diagnostics.Trace.TraceWarning(nameof(LoadAsync));
        }
        finally
        {
            _busy = false;
            Raise(nameof(ControlsEnabled));
        }
    }

    /// <summary>
    /// Takes a settings snapshot this page did not ask for -- the window
    /// hands one over whenever the daemon reports a change, so the page and
    /// the rail badge cannot come to disagree.
    /// </summary>
    public void Fill(DaemonSettingsSnapshot? settings)
    {
        if (settings is null)
        {
            return;
        }

        _on = settings.PrivateInferenceOn;
        _state = PrivateInferenceState.From(settings.PrivateInferenceReport);
        RaiseEverythingDerived();
    }

    /// <summary>
    /// Writes the switch and renders from the daemon's echo, never
    /// optimistically: the echo carries the only thing that knows whether the
    /// listener actually started. The marker rides along, so a contributor
    /// who found the switch here is not asked the question later.
    /// </summary>
    /// <remarks>
    /// Every path out of this method either wrote the value or corrected it.
    /// The toggle is bound one-way, so it has already moved to wherever the
    /// contributor dragged it, and only a change notification puts it back.
    /// </remarks>
    public async Task SetAsync(bool on)
    {
        if (!_loaded || _busy || _copy is null)
        {
            Raise(nameof(Enabled));
            return;
        }

        _busy = true;
        Raise(nameof(ControlsEnabled));
        try
        {
            DaemonResponse response = await _host
                .CallAsync(
                    DaemonProtocol.Methods.SetSettings,
                    PrivateInferenceSurface.SerializeSwitch(on))
                .ConfigureAwait(true);
            if (response.ResultAs<DaemonSettingsSnapshot>() is { } settings &&
                PrivateInferenceSurface.WriteConfirmed(on, settings))
            {
                Notice = string.Empty;
                Fill(settings);
            }
            else
            {
                // The reply did not confirm the full write. The state LINE is
                // left to the next refresh -- nothing here invents a sentence
                // -- but the SWITCH is snapped back to what the daemon last
                // reported, because a stuck switch is a claim.
                Notice = _copy.WriteUnconfirmed;
                Raise(nameof(Enabled));
            }
        }
        catch
        {
            System.Diagnostics.Trace.TraceWarning(nameof(SetAsync));
            Notice = _copy.WriteUnconfirmed;
            Raise(nameof(Enabled));
        }
        finally
        {
            _busy = false;
            Raise(nameof(ControlsEnabled));
        }
    }

    private void RaiseEverythingDerived()
    {
        Raise(nameof(Enabled));
        Raise(nameof(StateText));
        Raise(nameof(ServingText));
        Raise(nameof(HasServingText));
        Raise(nameof(IsWorking));
        Raise(nameof(StateIsNeutral));
        Raise(nameof(StateIsHeld));
        Raise(nameof(StateIsClear));
        Raise(nameof(StateIsAttention));
        Raise(nameof(StateIsRefused));
    }

    private void Raise(string? name) =>
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
}
