using System;
using TraceCommons.Interop;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.Windows.AppLifecycle;

namespace TraceCommons.App;

public static class Program
{
    private static readonly ActivationInbox<AppActivationArguments> Activations = new();

    [STAThread]
    private static int Main(string[] args)
    {
        WinRT.ComWrappersSupport.InitializeComWrappers();
        // Subscribe before claiming the key: another process can redirect as
        // soon as registration succeeds, before the XAML window exists.
        AppInstance.GetCurrent().Activated += (_, activation) => Activations.Enqueue(activation);
        try
        {
            var holder = AppInstance.FindOrRegisterForKey("TraceCommons.Main");
            if (!holder.IsCurrent && Redirect(holder)) return 0;
        }
        catch (Exception)
        {
            // Preserve the existing daemon-lock refusal UI on SDK failure.
            // Activation credentials and exception messages are never logged.
        }
        Application.Start(initialization =>
        {
            SynchronizationContext.SetSynchronizationContext(
                new DispatcherQueueSynchronizationContext(DispatcherQueue.GetForCurrentThread()));
            _ = new App();
        });
        return 0;
    }

    internal static void ReceiveActivations(Action<AppActivationArguments> receive) => Activations.Attach(receive);

    private static bool Redirect(AppInstance holder)
    {
        var activation = AppInstance.GetCurrent().GetActivatedEventArgs();
        var completed = new EventWaitHandle(false, EventResetMode.ManualReset);
        var redirect = Task.Run(async () =>
        {
            try { await holder.RedirectActivationToAsync(activation); return true; }
            catch (Exception) { return false; }
            finally { completed.Set(); }
        });
        try
        {
            // Pump COM on the STA thread while the MTA performs redirection.
            // A stale holder must not hang a launch indefinitely.
            uint result = CoWaitForMultipleObjects(0, 30000, 1,
                new[] { completed.SafeWaitHandle.DangerousGetHandle() }, out _);
            return result == 0 && redirect.GetAwaiter().GetResult();
        }
        finally
        {
            // On timeout the worker still owns the event until it signals.
            // Dispose only after both the native wait and worker are finished.
            _ = redirect.ContinueWith(_ => completed.Dispose(), TaskScheduler.Default);
        }
    }

    [DllImport("ole32.dll")]
    private static extern uint CoWaitForMultipleObjects(
        uint flags, uint milliseconds, uint count, IntPtr[] handles, out uint index);
}
