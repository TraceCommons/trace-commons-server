using System.Threading.Tasks;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using TraceCommons.App.ViewModels;
using TraceCommons.Interop;

namespace TraceCommons.App.Controls;

/// <summary>
/// The model-calls destination's markup and wiring.
///
/// Thin by design, like <see cref="SettingsView"/> and <see cref="HistoryView"/>:
/// every word and every decision lives in
/// <see cref="PrivateInferenceViewModel"/>, and the shapes it depends on live
/// one layer further down in TraceCommons.Interop so they can be tested off
/// Windows.
/// </summary>
public sealed partial class PrivateInferenceView : UserControl
{
    public PrivateInferenceView(DaemonHost host)
    {
        InitializeComponent();

        ViewModel = new PrivateInferenceViewModel(host);
        Loaded += OnFirstLoaded;
    }

    public PrivateInferenceViewModel ViewModel { get; }

    /// <summary>
    /// Reads the switch once the page is on screen rather than in the
    /// constructor, matching <see cref="HistoryView"/>: an IPC call that runs
    /// before the pane has been laid out reads as the nav click having done
    /// nothing.
    /// </summary>
    private async void OnFirstLoaded(object sender, RoutedEventArgs e)
    {
        Loaded -= OnFirstLoaded;
        await ViewModel.LoadAsync();
    }

    /// <summary>
    /// Hands a settings snapshot the window already has to the page, so a
    /// change made in the tray or in another shell lands here without a
    /// second read.
    /// </summary>
    public void Fill(DaemonSettingsSnapshot? settings) => ViewModel.Fill(settings);

    /// <summary>Flips the switch from outside the page, for the accelerator and the tray.</summary>
    public Task ToggleAsync() => ViewModel.SetAsync(!ViewModel.Enabled);

    /// <summary>
    /// The switch itself.
    ///
    /// Guarded against the programmatic fill: the toggle is bound one-way to
    /// the view model, so re-rendering it after a write raises Toggled again
    /// and would echo the value straight back at the daemon.
    /// </summary>
    private async void OnToggled(object sender, RoutedEventArgs e)
    {
        if (sender is not ToggleSwitch toggle || toggle.IsOn == ViewModel.Enabled)
        {
            return;
        }

        await ViewModel.SetAsync(toggle.IsOn);
    }
}
