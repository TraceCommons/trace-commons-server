using System;
using System.Runtime.InteropServices;

namespace TraceCommons.App;

/// <summary>Operating-system imports used by application activation.</summary>
internal static class Win32NativeMethods
{
    [DllImport("ole32.dll")]
    internal static extern uint CoWaitForMultipleObjects(
        uint flags, uint milliseconds, uint count, IntPtr[] handles, out uint index);
}
