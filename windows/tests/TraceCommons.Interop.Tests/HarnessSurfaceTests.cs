using System.Collections.Generic;
using TraceCommons.Interop;
using Xunit;

namespace TraceCommons.Interop.Tests;

/// <summary>
/// The harness list, the plan and the commit, decoded.
///
/// <para>
/// These mirror the macOS shell's cases on purpose: two shells decoding the
/// same payload by different rules would produce two behaviours nobody could
/// compare. Where the plan document's snippets quote an older field set
/// (<c>wired</c>, <c>facades</c>, <c>noop</c>, <c>digest</c>), the cases below
/// quote what <c>daemon/harness.rs</c> actually puts on the wire.
/// </para>
/// </summary>
public class HarnessSurfaceTests
{
    private const string OneRow =
        """
        {"catalog_present":false,"destination_port":8787,
         "harnesses":[{"id":"claude","name":"Claude Code","installed":true,
           "connected":false,"config_path":"C:\\Users\\x\\.claude\\settings.json",
           "connect_command":"ironwire connect claude","family":"anthropic",
           "state":"not_connected","last_call_at":null,
           "can_connect":true,"can_disconnect":false}],
         "activity":{"readable":true,"window_hours":24,"last_call_at":null,"families":[]}}
        """;

    [Fact]
    public void AMalformedRowIsDroppedRatherThanGuessed()
    {
        Assert.Empty(HarnessSurface.ParseListing("""{"harnesses":[{"name":"x"}]}""").Harnesses);
    }

    [Fact]
    public void AMalformedPayloadIsAnEmptyListRatherThanAThrow()
    {
        Assert.Empty(HarnessSurface.ParseListing("{not json").Harnesses);
        Assert.Empty(HarnessSurface.ParseListing(null).Harnesses);
        Assert.Empty(HarnessSurface.ParseListing("  ").Harnesses);
    }

    [Fact]
    public void TheRowCarriesTheFileItWouldChange()
    {
        HarnessListing listing = HarnessSurface.ParseListing(OneRow);
        Assert.Single(listing.Harnesses);
        Assert.Equal(@"C:\Users\x\.claude\settings.json", listing.Harnesses[0].ConfigPath);
        Assert.Equal("ironwire connect claude", listing.Harnesses[0].ConnectCommand);
        Assert.Equal("Claude Code", listing.Harnesses[0].Name);
    }

    /// <summary>
    /// The catalog channel is inert in this build, so the list is the tools
    /// this app was compiled knowing about. A surface that hid that would
    /// imply the machine has only these.
    /// </summary>
    [Fact]
    public void TheListSaysWhetherItCameFromACatalog()
    {
        Assert.False(HarnessSurface.ParseListing(OneRow).CatalogPresent);
    }

    [Fact]
    public void ANoopPlanIsDistinctFromAChange()
    {
        HarnessPlan? noop = HarnessSurface.ParsePlan(
            """{"id":"claude","action":"connect","outcome":"noop","plan_id":null,"path":"/p","changes":[],"occupied":[]}""");
        Assert.Equal(HarnessPlanOutcome.Noop, noop!.Outcome);
        Assert.False(noop.IsCommittable);

        HarnessPlan? real = HarnessSurface.ParsePlan(
            """{"id":"claude","action":"connect","outcome":"changes","plan_id":"a5f0e4a7-2a1f-4f3e-8b1e-0f6c1d2b3a44","path":"/p","changes":["set a thing"],"occupied":[]}""");
        Assert.Equal(HarnessPlanOutcome.Changes, real!.Outcome);
        Assert.True(real.IsCommittable);
        Assert.Equal(new[] { "set a thing" }, real.Changes);
        Assert.Equal("a5f0e4a7-2a1f-4f3e-8b1e-0f6c1d2b3a44", real.PlanId);
    }

    /// <summary>
    /// An unparseable config is a refusal that needs a human, not "nothing to
    /// change". A shell that collapsed the two would tell a contributor with a
    /// broken settings file that everything was fine.
    /// </summary>
    [Fact]
    public void ARefusedFileIsNotANoop()
    {
        HarnessPlan? refused = HarnessSurface.ParsePlan(
            """{"id":"claude","action":"connect","outcome":"unparseable","plan_id":null,"path":"/p","changes":[],"occupied":[]}""");
        Assert.Equal(HarnessPlanOutcome.Unparseable, refused!.Outcome);
        Assert.NotEqual(HarnessPlanOutcome.Noop, refused.Outcome);
        Assert.False(refused.IsCommittable);
    }

    /// <summary>
    /// A plan id is minted for a committable plan and for nothing else, so a
    /// shell cannot construct a write out of an outcome that refused one.
    /// </summary>
    [Fact]
    public void OnlyAChangeCarriesAPlanId()
    {
        foreach (string outcome in new[] { "noop", "unparseable", "not_installed", "entry_unusable", "no_config_path" })
        {
            HarnessPlan? plan = HarnessSurface.ParsePlan(
                $$"""{"id":"claude","action":"connect","outcome":"{{outcome}}","plan_id":null,"path":null,"changes":[],"occupied":[]}""");
            Assert.False(plan!.IsCommittable);
            Assert.Null(plan.PlanId);
        }
    }

    /// <summary>An outcome this build never heard of is not committable.</summary>
    [Fact]
    public void AnOutcomeFromALaterDaemonIsNotCommittable()
    {
        HarnessPlan? plan = HarnessSurface.ParsePlan(
            """{"id":"claude","action":"connect","outcome":"a_thing_invented_later","plan_id":"a5f0e4a7-2a1f-4f3e-8b1e-0f6c1d2b3a44","path":"/p","changes":[],"occupied":[]}""");
        Assert.Equal(HarnessPlanOutcome.Unknown, plan!.Outcome);
        Assert.False(plan.IsCommittable);
    }

    [Fact]
    public void AnOccupiedSlotSurvivesToTheScreen()
    {
        HarnessPlan? plan = HarnessSurface.ParsePlan(
            """{"id":"claude","action":"connect","outcome":"noop","plan_id":null,"path":"/p","changes":[],"occupied":[{"slot":"env.ANTHROPIC_BASE_URL","current":"https://theirs.example"}]}""");
        Assert.Equal("env.ANTHROPIC_BASE_URL", plan!.Occupied[0].Slot);
        Assert.Equal("https://theirs.example", plan.Occupied[0].Current);
    }

    /// <summary>
    /// Occupied rides alongside the outcome; it is not one. A plan can carry
    /// changes AND a slot that was left alone.
    /// </summary>
    [Fact]
    public void AnOccupiedSlotRidesAlongsideAChange()
    {
        HarnessPlan? plan = HarnessSurface.ParsePlan(
            """{"id":"claude","action":"connect","outcome":"changes","plan_id":"a5f0e4a7-2a1f-4f3e-8b1e-0f6c1d2b3a44","path":"/p","changes":["set a thing"],"occupied":[{"slot":"env.ANTHROPIC_BASE_URL","current":"https://theirs.example"}]}""");
        Assert.True(plan!.IsCommittable);
        Assert.Single(plan.Occupied);
    }

    [Fact]
    public void ANotInstalledToolCannotBeConnected()
    {
        var row = new HarnessRow { Id = "codex", Name = "Codex", Installed = false };
        Assert.False(row.CanConnect);
    }

    /// <summary>
    /// Uninstalling a coding tool does not remove the line we put in its
    /// config file, so a connected tool may always be disconnected.
    /// </summary>
    [Fact]
    public void AConnectedToolMayBeDisconnectedEvenUninstalled()
    {
        Assert.True(HarnessSurface.ActionAvailable(HarnessSurface.Disconnect, installed: false, connected: true));
        Assert.False(HarnessSurface.ActionAvailable(HarnessSurface.Connect, installed: false, connected: true));
        Assert.False(HarnessSurface.ActionAvailable("rewire", installed: true, connected: true));
    }

    /// <summary>The daemon's own answer and the ABI's agree, per row.</summary>
    [Fact]
    public void TheRowsAnswerMatchesTheSharedBranchTable()
    {
        HarnessRow row = HarnessSurface.ParseListing(OneRow).Harnesses[0];
        Assert.Equal(
            HarnessSurface.ActionAvailable(HarnessSurface.Connect, row.Installed, row.Connected),
            row.CanConnect);
        Assert.Equal(
            HarnessSurface.ActionAvailable(HarnessSurface.Disconnect, row.Installed, row.Connected),
            row.CanDisconnect);
    }

    /// Only the clear tone may be painted as working, on a harness row as anywhere.
    [Fact]
    public void OnlyAnAnsweringHarnessReadsAsWorking()
    {
        Assert.True(HarnessSurface.ReadsAsWorking(HarnessSurface.State("answering")));
        foreach (string label in new[]
        {
            "not_connected", "connected_no_calls", "activity_shared", "unknown",
            "", "a_state_from_a_later_daemon",
        })
        {
            Assert.False(HarnessSurface.ReadsAsWorking(HarnessSurface.State(label)));
        }
    }

    /// <summary>
    /// A call arrived in this tool's protocol family and more than one
    /// connected tool speaks it. The sentence that says a call was answered
    /// is still true of this computer; what may not happen is painting the
    /// row as this tool working.
    /// </summary>
    [Fact]
    public void ASharedFamilyIsItsOwnStateAndNotAFlavourOfAnswering()
    {
        Assert.Equal(HarnessState.ActivityShared, HarnessSurface.State("activity_shared"));
        Assert.NotEqual(HarnessState.Answering, HarnessSurface.State("activity_shared"));
    }

    [Fact]
    public void AStateFromALaterDaemonIsUnknown()
    {
        Assert.Equal(HarnessState.Unknown, HarnessSurface.State("a_state_from_a_later_daemon"));
        Assert.Equal(HarnessState.Unknown, HarnessSurface.State(null));
    }

    /// <summary>
    /// The shell cannot construct a write. A commit carries the plan id the
    /// daemon minted and nothing else -- no id, no action, no path.
    /// </summary>
    [Fact]
    public void ACommitCarriesOnlyThePlanTheDaemonMinted()
    {
        string body = HarnessSurface.SerializeCommit("a5f0e4a7-2a1f-4f3e-8b1e-0f6c1d2b3a44");
        Assert.Equal("""{"plan_id":"a5f0e4a7-2a1f-4f3e-8b1e-0f6c1d2b3a44"}""", body);
    }

    [Fact]
    public void APlanRequestNamesTheToolAndTheDirection()
    {
        Assert.Equal(
            """{"id":"claude","action":"connect"}""",
            HarnessSurface.SerializePlan("claude", HarnessSurface.Connect));
        Assert.Equal(
            """{"id":"codex","action":"disconnect"}""",
            HarnessSurface.SerializePlan("codex", HarnessSurface.Disconnect));
    }

    /// <summary>
    /// A plan is single-use and short-lived. When the daemon has forgotten
    /// one, the shell re-fetches rather than treating it as a failed write.
    /// </summary>
    [Fact]
    public void AForgottenPlanIsRecognisedRatherThanReportedAsAFailedWrite()
    {
        Assert.True(HarnessSurface.PlanIsStale(new DaemonError { Code = "unavailable", Message = "harness-plan-unknown" }));
        Assert.True(HarnessSurface.PlanIsStale(new DaemonError { Code = "unavailable", Message = "harness-config-changed" }));
        Assert.False(HarnessSurface.PlanIsStale(new DaemonError { Code = "unavailable", Message = "harness-commit-failed" }));
        Assert.False(HarnessSurface.PlanIsStale(null));
    }

    /// <summary>
    /// A connect cannot be planned when nothing answers here. The daemon says
    /// so by name rather than by an outcome, because it is a fact about this
    /// computer and not about the tool's file.
    /// </summary>
    [Fact]
    public void AConnectWithNoDestinationIsRecognised()
    {
        Assert.True(HarnessSurface.NoDestination(new DaemonError { Code = "bad-params", Message = "harness-no-destination" }));
        Assert.False(HarnessSurface.NoDestination(new DaemonError { Code = "bad-params", Message = "harness-unknown" }));
    }

    [Fact]
    public void ACommitAnswerCarriesTheFileAndTheCopyItPreserved()
    {
        HarnessCommit? commit = HarnessSurface.ParseCommit(
            """{"id":"claude","action":"connect","committed":true,"path":"/p","backup_path":"/p.ironwire-backup"}""");
        Assert.True(commit!.Committed);
        Assert.Equal("/p", commit.Path);
        Assert.Equal("/p.ironwire-backup", commit.BackupPath);
        Assert.Null(HarnessSurface.ParseCommit("{nope"));
    }

    /// <summary>
    /// Every sentence a harness row shows comes off the payload. Nothing here
    /// composes one, so an unknown state shows no sentence rather than a
    /// guessed one.
    /// </summary>
    [Fact]
    public void EveryStateSentenceComesFromThePayload()
    {
        PrivateInferenceCopy? copy = PrivateInferenceSurface.Copy();
        Assert.NotNull(copy);

        var expected = new Dictionary<HarnessState, string>
        {
            [HarnessState.NotConnected] = copy!.HarnessNotConnected,
            [HarnessState.ConnectedNoCalls] = copy.HarnessConnectedNothingSeen,
            [HarnessState.Answering] = copy.HarnessAnswering,
            [HarnessState.ActivityShared] = string.Empty,
            [HarnessState.Unknown] = string.Empty,
        };

        foreach (KeyValuePair<HarnessState, string> pair in expected)
        {
            Assert.Equal(pair.Value, HarnessSurface.StateSentence(pair.Key, copy));
        }
    }

    /// <summary>
    /// An unattributable call must never be described with the answering
    /// sentence.
    /// </summary>
    /// <remarks>
    /// The sentence reads "Answering. A call from <em>it</em> reached this
    /// computer and was answered here." The pronoun names the row's own tool.
    /// <see cref="HarnessState.ActivityShared"/> exists precisely because the
    /// call cannot be attributed to one tool of a shared family, so borrowing
    /// that sentence would assert the one thing the state says is unknown.
    ///
    /// Asserted against the sentence itself rather than against an empty
    /// string, so that adding a real ActivityShared sentence later satisfies
    /// this test while still failing if it reaches for the answering one.
    ///
    /// This mirrors the macOS surface, which returns nil for the same state.
    /// The two shells must not disagree about what an unattributable call is
    /// called.
    /// </remarks>
    [Fact]
    public void AnUnattributableCallNeverBorrowsTheAnsweringSentence()
    {
        PrivateInferenceCopy? copy = PrivateInferenceSurface.Copy();
        Assert.NotNull(copy);

        string shared = HarnessSurface.StateSentence(HarnessState.ActivityShared, copy!);
        Assert.NotEqual(copy!.HarnessAnswering, shared);
        Assert.False(HarnessSurface.ReadsAsWorking(HarnessState.ActivityShared));
    }
}
