using TraceCommons.Interop;
using Xunit;

namespace TraceCommons.Interop.Tests;

public sealed class CloseBehaviorTests
{
    [Theory]
    [InlineData(false, false, false, CloseRequestOutcome.AskToQuit)]
    [InlineData(false, true, false, CloseRequestOutcome.HideToTray)]
    [InlineData(false, false, true, CloseRequestOutcome.KeepConfirmationVisible)]
    [InlineData(false, true, true, CloseRequestOutcome.KeepConfirmationVisible)]
    [InlineData(true, false, false, CloseRequestOutcome.Quit)]
    [InlineData(true, true, false, CloseRequestOutcome.Quit)]
    [InlineData(true, false, true, CloseRequestOutcome.Quit)]
    [InlineData(true, true, true, CloseRequestOutcome.Quit)]
    public void CloseRequiresEitherAReachableTrayOrExplicitQuit(
        bool confirmed, bool trayPresent, bool confirmationPending, CloseRequestOutcome expected)
    {
        Assert.Equal(expected, CloseBehavior.OnWindowClose(confirmed, trayPresent, confirmationPending));
    }
}
