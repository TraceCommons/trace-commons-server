using System;
using System.Collections.Generic;

namespace TraceCommons.Interop;

/// <summary>Serializes early activation delivery until the UI dispatcher is available.</summary>
public sealed class ActivationInbox<T>
{
    private readonly object _gate = new();
    private readonly Queue<T> _pending = new();
    private Action<T>? _receive;
    private bool _draining;

    public void Enqueue(T activation)
    {
        lock (_gate)
        {
            _pending.Enqueue(activation);
            if (_receive is null || _draining) return;
            _draining = true;
        }
        Drain();
    }

    /// <summary>The callback must enqueue UI work and return without waiting for it.</summary>
    public void Attach(Action<T> receive)
    {
        ArgumentNullException.ThrowIfNull(receive);
        lock (_gate)
        {
            if (_receive is not null) throw new InvalidOperationException("activation-receiver-already-attached");
            _receive = receive;
            _draining = true;
        }
        Drain();
    }

    private void Drain()
    {
        while (true)
        {
            T activation;
            Action<T> receive;
            lock (_gate)
            {
                if (!_pending.TryDequeue(out activation!))
                {
                    _draining = false;
                    return;
                }
                receive = _receive!;
            }
            // A single drainer preserves ordering without holding the lock
            // across external code, including a callback that enqueues again.
            try { receive(activation); }
            catch
            {
                lock (_gate) _draining = false;
                throw;
            }
        }
    }
}
