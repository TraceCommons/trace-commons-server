using System;
using System.Collections.Generic;

namespace TraceCommons.Interop;

/// <summary>Serializes early activation delivery until the UI dispatcher is available.</summary>
public sealed class ActivationInbox<T>
{
    private readonly object _gate = new();
    private readonly Queue<T> _pending = new();
    private Action<T>? _receive;

    public void Enqueue(T activation)
    {
        lock (_gate)
        {
            if (_receive is null) _pending.Enqueue(activation);
            else _receive(activation);
        }
    }

    /// <summary>The callback must enqueue UI work and return without waiting for it.</summary>
    public void Attach(Action<T> receive)
    {
        lock (_gate)
        {
            if (_receive is not null) throw new InvalidOperationException("activation-receiver-already-attached");
            _receive = receive;
            while (_pending.TryDequeue(out var activation)) receive(activation);
        }
    }
}
