using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using TraceCommons.Interop;
using Xunit;

namespace TraceCommons.Interop.Tests;

public sealed class ActivationInboxTests
{
    [Fact]
    public async Task StartupAndConcurrentActivationsAreDeliveredExactlyOnce()
    {
        var inbox = new ActivationInbox<int>();
        var delivered = new List<int>();
        inbox.Enqueue(-2);
        inbox.Enqueue(-1);
        var producers = Task.Run(() => Parallel.For(0, 100, inbox.Enqueue));
        inbox.Attach(delivered.Add);
        await producers;
        Assert.Equal(new[] { -2, -1 }, delivered.Take(2));
        Assert.Equal(102, delivered.Count);
        Assert.Equal(Enumerable.Range(-2, 102), delivered.OrderBy(value => value));
        Assert.Throws<InvalidOperationException>(() => inbox.Attach(_ => { }));
    }

    [Fact]
    public void CallbackCanWaitForAnotherProducerWithoutHoldingTheInboxLock()
    {
        var inbox = new ActivationInbox<int>();
        var delivered = new List<int>();
        inbox.Enqueue(1);
        inbox.Attach(value =>
        {
            delivered.Add(value);
            if (value == 1)
            {
                Assert.True(Task.Run(() => inbox.Enqueue(2)).Wait(TimeSpan.FromSeconds(5)));
            }
        });
        Assert.Equal(new[] { 1, 2 }, delivered);
    }

    [WindowsFact]
    public void NativeWindowsRedirectedArgumentsUsePlatformQuoting()
    {
        Assert.Equal("one", DeepLink.InviteFromCommandLine("\"C:\\Program Files\\Trace Commons.exe\" \"tracecommons://enroll?invite=one\""));
        Assert.Null(DeepLink.InviteFromCommandLine("TraceCommons.exe https://enroll?invite=wrong"));
        Assert.Null(DeepLink.InviteFromCommandLine("TraceCommons.exe --ordinary-launch"));
        Assert.Null(DeepLink.InviteFromCommandLine(" "));
    }
}

public sealed class WindowsFactAttribute : FactAttribute
{
    public WindowsFactAttribute()
    {
        if (!OperatingSystem.IsWindows())
        {
            Skip = "Requires the Windows command-line parser; exercised by Windows CI.";
        }
    }
}
