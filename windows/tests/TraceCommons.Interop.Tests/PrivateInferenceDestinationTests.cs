using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using TraceCommons.Interop;
using Xunit;

namespace TraceCommons.Interop.Tests;

/// <summary>
/// Model calls as a top-level destination: the rail item, the page, the tray
/// entry and the two rules they all hang on.
///
/// <para>
/// The first rule is that only a clear tone may be painted as working. A rail
/// badge and a tray glyph both invite a green dot, and painting a refusal as
/// "on" is the fail-open this whole surface exists to prevent.
/// </para>
/// <para>
/// The second is that the indicator derives from the tone and never from the
/// settings boolean. The switch says what was ASKED FOR; the indicator says
/// what is true, and the two disagree exactly when it matters -- the switch
/// on, the listener refused to start.
/// </para>
/// </summary>
public class PrivateInferenceDestinationTests
{
    private static PrivateInferenceCopy Copy()
    {
        PrivateInferenceCopy? copy = PrivateInferenceSurface.Copy();
        Assert.NotNull(copy);
        return copy!;
    }

    private static PrivateInferenceState State(string label, ushort? port = null) =>
        new(label, port);

    /// <summary>
    /// One predicate, and only one value satisfies it. Every indicator in
    /// this shell asks this rather than deciding for itself.
    /// </summary>
    [Fact]
    public void OnlyTheClearToneReadsAsWorking()
    {
        Assert.True(PrivateInferenceTone.Clear.ReadsAsWorking());
        foreach (PrivateInferenceTone tone in new[]
        {
            PrivateInferenceTone.Neutral,
            PrivateInferenceTone.Held,
            PrivateInferenceTone.Attention,
            PrivateInferenceTone.Refused,
        })
        {
            Assert.False(tone.ReadsAsWorking(), $"{tone} must not be painted as working");
        }
    }

    /// <summary>
    /// A value from a later daemon reaches the predicate as an int, and it
    /// must not arrive as the working light.
    /// </summary>
    [Fact]
    public void AnUnrecognisedAbiValueDoesNotReadAsWorking()
    {
        foreach (int stranger in new[]
        {
            0, 1, 2, 3, 10, 14, 20, 25, 26, 99, -1, int.MaxValue, int.MinValue,
        })
        {
            Assert.False(
                PrivateInferenceSurface.FromAbiTone(stranger).ReadsAsWorking(),
                $"the unrecognised ABI value {stranger} must not be painted as working");
        }
    }

    /// <summary>
    /// Every state the daemon reports, through the whole chain a rail badge
    /// and a tray glyph actually take. Only the running listener with
    /// somewhere to send lights up.
    /// </summary>
    [Fact]
    public void OnlyARunningListenerLightsTheIndicator()
    {
        Assert.True(PrivateInferenceSurface.Tone(State("running")).ReadsAsWorking());
        foreach (string label in new[]
        {
            "", "off", "stopping", "running_no_backends", "running_elsewhere",
            "port_in_use", "start_failed", "crashed", "a_state_from_a_later_daemon",
        })
        {
            Assert.False(
                PrivateInferenceSurface.Tone(State(label)).ReadsAsWorking(),
                $"{label} must not read as working");
        }
    }

    /// <summary>
    /// The tray entry is computed from the reported state, never from the
    /// switch. The two are handed in separately here precisely so this can be
    /// asserted: switch on, listener refused, indicator dark.
    /// </summary>
    [Fact]
    public void TheTrayIndicatorFollowsTheStateAndNotTheSwitch()
    {
        PrivateInferenceCopy copy = Copy();
        foreach (string refused in new[] { "port_in_use", "start_failed", "crashed" })
        {
            PrivateInferenceTrayEntry entry =
                PrivateInferenceTrayEntry.For(copy, State(refused), on: true);
            Assert.True(entry.Available);
            Assert.True(entry.On);
            Assert.False(
                entry.ReadsAsWorking,
                $"the tray painted {refused} as working because the switch was on");
        }

        PrivateInferenceTrayEntry running =
            PrivateInferenceTrayEntry.For(copy, State("running", 8463), on: true);
        Assert.True(running.ReadsAsWorking);

        // And the mirror image: the switch off while a listener this app
        // started is still coming down. Nothing claims to be working.
        Assert.False(
            PrivateInferenceTrayEntry.For(copy, State("stopping"), on: false).ReadsAsWorking);
    }

    /// <summary>
    /// Every word on the tray entry is the payload's. None is spelled here.
    /// </summary>
    [Fact]
    public void TheTrayEntryTakesEveryWordFromThePayload()
    {
        PrivateInferenceCopy copy = Copy();
        PrivateInferenceTrayEntry entry =
            PrivateInferenceTrayEntry.For(copy, State("running", 8463), on: true);

        Assert.Equal(copy.Destination, entry.Label);
        Assert.Equal(copy.SettingsToggle, entry.ToggleText);
        Assert.Equal(
            PrivateInferenceSurface.StateLine(State("running", 8463), copy), entry.StateText);
    }

    /// <summary>
    /// No payload, no entry. The same stance the settings card takes: a menu
    /// row with a switch and no sentence beside it is the shape that says
    /// "on" over a listener that refused to start.
    /// </summary>
    [Fact]
    public void AMissingPayloadLeavesTheTrayEntryOut()
    {
        PrivateInferenceTrayEntry entry =
            PrivateInferenceTrayEntry.For(null, State("running"), on: true);
        Assert.False(entry.Available);
        Assert.False(entry.ReadsAsWorking);
        Assert.Equal(string.Empty, entry.Label);
        Assert.Equal(string.Empty, entry.StateText);
        Assert.Equal(string.Empty, entry.ToggleText);
    }

    /// <summary>
    /// The rail label is short enough to sit in a 184px rail beside the other
    /// three, and it is the Rust's word rather than this shell's.
    /// </summary>
    [Fact]
    public void TheRailLabelArrivesFromTheRust()
    {
        PrivateInferenceCopy copy = Copy();
        Assert.False(string.IsNullOrWhiteSpace(copy.Destination));
        Assert.False(string.IsNullOrWhiteSpace(copy.Subtitle));
        Assert.True(
            copy.Destination.Length <= 24,
            $"the rail label does not fit beside the other three: {copy.Destination}");
    }

    private static string ShellSource(string relativePath)
    {
        string path = Path.Combine(
            AppContext.BaseDirectory, "shell-source", relativePath + ".txt");
        Assert.True(File.Exists(path), $"{relativePath} was not copied to {path}");
        return File.ReadAllText(path).Replace("\r\n", "\n", StringComparison.Ordinal);
    }

    /// <summary>
    /// The rail item's label is bound, never typed. The label is the one word
    /// a contributor navigates by, and a shell that spelled it itself would
    /// go on spelling the old one after a rename in the Rust.
    /// </summary>
    [Fact]
    public void TheRailItemBindsItsLabelRatherThanSpellingIt()
    {
        string markup = ShellSource("TraceCommons.App/MainWindow.xaml");
        Assert.Contains(
            "ViewModel.PrivateInferenceDestination", markup, StringComparison.Ordinal);
        Assert.Contains("OnShowPrivateInference", markup, StringComparison.Ordinal);
    }

    /// <summary>
    /// Not one surface in this shell spells the destination's name.
    ///
    /// The Rust owns it, and the reason it is that word rather than the
    /// setting's internal one is that turning this on does not make a call
    /// private: it moves where the call is answered, and the call still goes
    /// on to whoever answers it. A shell that typed either phrase would be
    /// making a promise the Rust deliberately does not.
    /// </summary>
    [Fact]
    public void NoShellSourceSpellsTheDestinationOrCallsItPrivate()
    {
        string root = Path.Combine(AppContext.BaseDirectory, "shell-source");
        Assert.True(Directory.Exists(root), $"the shell sources were not copied to {root}");

        var offenders = new List<string>();
        foreach (string file in Directory.EnumerateFiles(root, "*.txt", SearchOption.AllDirectories))
        {
            string relative = Path.GetRelativePath(root, file).Replace('\\', '/');
            string source = File.ReadAllText(file);

            // Comments are prose about this surface and quote it on purpose.
            string uncommented = string.Join(
                "\n",
                source.Replace("\r\n", "\n", StringComparison.Ordinal).Split('\n')
                    .Where(line => !line.TrimStart().StartsWith("//", StringComparison.Ordinal)));
            uncommented = Regex.Replace(
                uncommented, "<!--.*?-->", string.Empty, RegexOptions.Singleline);

            foreach (string forbidden in new[]
            {
                "Model calls", "Private inference", "private inference",
            })
            {
                if (uncommented.Contains(forbidden, StringComparison.Ordinal))
                {
                    offenders.Add($"{relative} spells \"{forbidden}\"");
                }
            }
        }

        Assert.True(
            offenders.Count == 0,
            "The destination's name comes from private_inference_copy.rs across the ABI.\n"
            + string.Join("\n", offenders));
    }

    /// <summary>
    /// The rail badge, the page's own status line and the tray glyph all
    /// derive from the tone. None of them reads the switch.
    /// </summary>
    [Fact]
    public void NoIndicatorInThisShellFollowsTheSwitch()
    {
        string viewModel = ShellSource("TraceCommons.App/ViewModels/MainViewModel.cs");
        Assert.Contains(
            "PrivateInferenceSurface.Tone(_privateInferenceState)",
            viewModel,
            StringComparison.Ordinal);
        Assert.Contains("ReadsAsWorking()", viewModel, StringComparison.Ordinal);

        // The rail badge's own property, and what it is allowed to read.
        string badge = Property(viewModel, "public bool PrivateInferenceIsWorking");
        Assert.DoesNotContain("_privateInferenceOn", badge, StringComparison.Ordinal);
        Assert.DoesNotContain("PrivateInferenceOn", badge, StringComparison.Ordinal);

        // And the markup: the rail badge's Visibility is the tone-derived
        // property, never the switch.
        string markup = ShellSource("TraceCommons.App/MainWindow.xaml");
        Assert.Contains("ViewModel.PrivateInferenceIsWorking", markup, StringComparison.Ordinal);
        Assert.DoesNotContain(
            "x:Bind ViewModel.PrivateInferenceOn", markup, StringComparison.Ordinal);
    }

    /// <summary>
    /// The destination page draws the shared sentences and picks its colour
    /// from the tone, exactly as the settings card does.
    /// </summary>
    [Fact]
    public void TheDestinationPageIsDrawnFromTheSharedSentences()
    {
        string markup = ShellSource("TraceCommons.App/Controls/PrivateInferenceView.xaml");

        foreach (string bound in new[]
        {
            "ViewModel.Title",
            "ViewModel.Subtitle",
            "ViewModel.Exposure",
            "ViewModel.ToggleText",
            "ViewModel.StateText",
            "ViewModel.AppliesAtOnce",
            "ViewModel.StateIsRefused",
        })
        {
            Assert.Contains(bound, markup, StringComparison.Ordinal);
        }

        // Every rendered string on the page is a binding, never a literal.
        foreach (Match match in Regex.Matches(markup, "(Text|Content|Header)=\"([^\"]*)\""))
        {
            Assert.StartsWith("{x:Bind", match.Groups[2].Value, StringComparison.Ordinal);
        }

        string viewModel = ShellSource("TraceCommons.App/ViewModels/PrivateInferenceViewModel.cs");
        foreach (string sourced in new[]
        {
            "_copy?.SettingsTitle",
            "_copy?.Subtitle",
            "_copy?.OfferExposure",
            "_copy?.SettingsToggle",
            "_copy?.SettingsAppliesAtOnce",
        })
        {
            Assert.Contains(sourced, viewModel, StringComparison.Ordinal);
        }

        Assert.Contains(
            "PrivateInferenceSurface.Tone(_state)", viewModel, StringComparison.Ordinal);
        Assert.DoesNotContain("StateText ==", viewModel, StringComparison.Ordinal);
        Assert.DoesNotContain("StateText.Contains", viewModel, StringComparison.Ordinal);
        Assert.DoesNotContain("StateText.StartsWith", viewModel, StringComparison.Ordinal);
    }

    /// <summary>
    /// Not one string literal in the destination's view model. A paraphrase
    /// would go in as a fallback beside a payload read -- friendlier, wrong,
    /// and passing every behavioural test in this file.
    /// </summary>
    [Fact]
    public void NoWordingIsAuthoredInTheDestinationViewModel()
    {
        string source = ShellSource("TraceCommons.App/ViewModels/PrivateInferenceViewModel.cs");
        string uncommented = string.Join(
            "\n",
            source.Split('\n')
                .Where(line => !line.TrimStart().StartsWith("//", StringComparison.Ordinal)));

        foreach (Match match in Regex.Matches(uncommented, "\"([^\"\\\\]|\\\\.)*\""))
        {
            Assert.Fail(
                $"{match.Value} is a string literal in the destination's view model. Every "
                + "sentence on this surface comes from private_inference_copy.rs across the ABI.");
        }
    }

    /// <summary>
    /// The four in-app shortcuts, and no global hotkey.
    ///
    /// A system-wide hotkey is a different thing entirely -- it steals a key
    /// combination from every other application on the machine -- and it was
    /// cut deliberately. RegisterHotKey appearing anywhere in this shell is
    /// that cut being quietly undone.
    /// </summary>
    [Fact]
    public void TheShortcutsAreInAppAndThereIsNoGlobalHotkey()
    {
        string markup = ShellSource("TraceCommons.App/MainWindow.xaml");
        foreach (string key in new[] { "Number1", "Number2", "Number3", "Number4" })
        {
            Assert.Contains($"Key=\"{key}\"", markup, StringComparison.Ordinal);
        }

        Assert.Contains("KeyboardAccelerator", markup, StringComparison.Ordinal);
        Assert.Contains("OnTogglePrivateInferenceAccelerator", markup, StringComparison.Ordinal);

        // Comments are stripped first: the prose beside the accelerators
        // names the API it is deliberately not using.
        string root = Path.Combine(AppContext.BaseDirectory, "shell-source");
        foreach (string file in Directory.EnumerateFiles(root, "*.txt", SearchOption.AllDirectories))
        {
            string source = File.ReadAllText(file).Replace("\r\n", "\n", StringComparison.Ordinal);
            string uncommented = string.Join(
                "\n",
                source.Split('\n')
                    .Where(line => !line.TrimStart().StartsWith("//", StringComparison.Ordinal)));
            uncommented = Regex.Replace(
                uncommented, "<!--.*?-->", string.Empty, RegexOptions.Singleline);

            Assert.DoesNotContain("RegisterHotKey", uncommented, StringComparison.Ordinal);
        }
    }

    /// <summary>
    /// The tray offers the destination and the switch, and it asks the shared
    /// model for both rather than composing them itself.
    /// </summary>
    [Fact]
    public void TheTrayEntryIsBuiltFromTheSharedModel()
    {
        string tray = ShellSource("TraceCommons.App/TrayIcon.cs");
        Assert.Contains("PrivateInferenceTrayEntry", tray, StringComparison.Ordinal);
        Assert.Contains("ReadsAsWorking", tray, StringComparison.Ordinal);
        Assert.Contains("PrivateInferenceToggleRequested", tray, StringComparison.Ordinal);
        Assert.Contains("PrivateInferenceRequested", tray, StringComparison.Ordinal);

        string window = ShellSource("TraceCommons.App/MainWindow.xaml.cs");
        Assert.Contains(
            "_tray.PrivateInferenceToggleRequested +=", window, StringComparison.Ordinal);
        Assert.Contains("_tray.PrivateInferenceRequested +=", window, StringComparison.Ordinal);
    }

    /// <summary>
    /// The body of one property, from its signature to the line that closes
    /// it. Crude on purpose, matching <c>PrivateInferenceTests.MethodBody</c>:
    /// this suite cannot compile the C# it is reading, and a parser here would
    /// be a second thing to get wrong.
    /// </summary>
    private static string Property(string source, string signature)
    {
        int start = source.IndexOf(signature, StringComparison.Ordinal);
        Assert.True(start >= 0, $"{signature} is gone");
        int end = source.IndexOf(";\n", start, StringComparison.Ordinal);
        Assert.True(end > start, $"{signature} does not close");
        return source[start..end];
    }
}
