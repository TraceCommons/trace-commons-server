namespace TraceCommons.Interop;

public enum CloseRequestOutcome
{
    Quit,
    HideToTray,
    AskToQuit,
    KeepConfirmationVisible,
}

/// <summary>Keep window dismissal separate from stopping the hosted daemon.</summary>
public static class CloseBehavior
{
    public static CloseRequestOutcome OnWindowClose(bool quitConfirmed, bool trayPresent, bool confirmationPending)
    {
        if (quitConfirmed)
        {
            return CloseRequestOutcome.Quit;
        }
        if (confirmationPending)
        {
            return CloseRequestOutcome.KeepConfirmationVisible;
        }
        return trayPresent ? CloseRequestOutcome.HideToTray : CloseRequestOutcome.AskToQuit;
    }
}
