using System;

namespace TraceCommons.Interop;

/// <summary>
/// The existing Settings action may restore manual review or ignore a project.
/// It never grants automatic-upload consent, including for unknown modes.
/// </summary>
public static class ProjectManualMode
{
    public static string? Next(string? currentMode) => currentMode switch
    {
        "auto_upload" or "ignore" => "ask",
        "ask" => "ignore",
        _ => null,
    };

    /// <summary>
    /// What to say after a write, given the daemon's answer and the mode read
    /// back from the daemon afterwards. Empty when the stored mode is the one
    /// that was asked for and nothing needs saying.
    ///
    /// The re-read is the point. A shell that sets its local mode from the
    /// value it sent is asserting a fact it never observed, and the failure it
    /// cannot see is exactly the one worth reporting: the daemon refused, or
    /// accepted and stored something else. <paramref name="persisted"/> is
    /// <c>null</c> when the row could not be found in the re-read at all --
    /// neither outcome is then known, so neither is claimed.
    /// </summary>
    /// <param name="writeFailed">Whether the <c>set_project_mode</c> call returned an error.</param>
    /// <param name="requested">The mode the shell asked for.</param>
    /// <param name="persisted">The mode the daemon reports storing, or null if it could not be read.</param>
    public static string NoticeFor(bool writeFailed, string? requested, string? persisted)
    {
        if (persisted is null)
        {
            // A failed write whose state cannot be re-read is still a failed
            // write: that much was observed, and it is the more specific of
            // the two sentences.
            return writeFailed ? WatchCopy.WriteFailed : WatchCopy.WriteUnconfirmed;
        }

        if (writeFailed || !string.Equals(requested, persisted, StringComparison.Ordinal))
        {
            return WatchCopy.WriteFailed;
        }

        return string.Empty;
    }
}
