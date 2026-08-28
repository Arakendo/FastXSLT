using System.Buffers.Binary;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

public sealed class NativeFastXsltClient : IDisposable
{
    private readonly NativeEngineHandle _engine;
    private readonly object _gate = new();

    private NativeFastXsltClient(NativeEngineHandle engine) => _engine = engine;

    public static NativeFastXsltClient Create(
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet)
    {
        if (NativeMethods.AbiVersion() != 1)
        {
            throw new InvalidOperationException("Unexpected native FastXSLT workbench ABI version.");
        }
        var sourceIdentityBytes = Encoding.UTF8.GetBytes(sourceIdentity);
        var stylesheetIdentityBytes = Encoding.UTF8.GetBytes(stylesheetIdentity);
        var outcome = NativeMethods.Create(
            sourceIdentityBytes,
            (nuint)sourceIdentityBytes.Length,
            source,
            (nuint)source.Length,
            stylesheetIdentityBytes,
            (nuint)stylesheetIdentityBytes.Length,
            stylesheet,
            (nuint)stylesheet.Length);
        if (NativeMethods.OutcomeKind(outcome) != 1)
        {
            throw ReadFailureAndRelease(outcome);
        }
        var engine = NativeMethods.OutcomeTakeEngine(outcome);
        if (engine == 0)
        {
            throw new InvalidDataException("Native creation outcome did not contain an engine.");
        }
        return new NativeFastXsltClient(new NativeEngineHandle(engine));
    }

    public string Transform(string requestIdentity)
    {
        lock (_gate)
        {
            ObjectDisposedException.ThrowIf(_engine.IsClosed, this);
            var request = Encoding.UTF8.GetBytes(requestIdentity);
            var outcome = NativeMethods.Transform(_engine.Value, request, (nuint)request.Length);
            return ReadTransformOutcome(outcome);
        }
    }

    public string TransformWithInvocationPolicy(
        string requestIdentity,
        bool cancellationRequested,
        ulong maximumXsltInstructions)
    {
        lock (_gate)
        {
            ObjectDisposedException.ThrowIf(_engine.IsClosed, this);
            var request = Encoding.UTF8.GetBytes(requestIdentity);
            var outcome = NativeMethods.TransformControlled(
                _engine.Value,
                request,
                (nuint)request.Length,
                cancellationRequested ? 1u : 0u,
                maximumXsltInstructions);
            return ReadTransformOutcome(outcome);
        }
    }

    public async Task<string> TransformAsync(
        string requestIdentity,
        CancellationToken cancellationToken,
        ulong maximumXsltInstructions = 1_000_000)
    {
        using var control = NativeControlHandle.Create(firstChargeBarrier: false);
        using var registration = cancellationToken.Register(
            static state => ((NativeControlHandle)state!).Cancel(),
            control);
        return await Task.Run(() => TransformWithControl(
            requestIdentity,
            control,
            maximumXsltInstructions));
    }

    public async Task<NativeActiveCancellationObservation> ExerciseActiveCancellationAsync(
        string requestIdentity,
        TimeSpan observationTimeout)
    {
        var target = NativeControlHandle.Create(firstChargeBarrier: true);
        using var unrelated = NativeControlHandle.Create(firstChargeBarrier: false);
        var invocation = Task.Run(() => TransformWithControl(
            requestIdentity,
            target,
            maximumXsltInstructions: 1_000_000));
        var waiting = System.Diagnostics.Stopwatch.StartNew();
        while (!target.FirstChargeObserved &&
               !invocation.IsCompleted &&
               waiting.Elapsed < observationTimeout)
        {
            await Task.Delay(1);
        }
        if (!target.FirstChargeObserved)
        {
            target.Cancel();
            try
            {
                _ = await invocation;
            }
            catch (NativeFastXsltException)
            {
            }
            target.Dispose();
            throw new TimeoutException("Native invocation did not reach its first charge barrier.");
        }

        unrelated.Cancel();
        var unrelatedSignalIgnored = !invocation.IsCompleted;
        var signal = System.Diagnostics.Stopwatch.StartNew();
        target.Cancel();
        NativeFastXsltException cancellation;
        try
        {
            _ = await invocation;
            throw new InvalidOperationException("Actively cancelled native invocation unexpectedly completed.");
        }
        catch (NativeFastXsltException failure)
        {
            cancellation = failure;
        }
        signal.Stop();
        target.Dispose();
        target.Dispose();
        return new NativeActiveCancellationObservation(
            cancellation,
            signal.Elapsed.TotalMilliseconds,
            FirstChargeObserved: true,
            UnrelatedSignalIgnored: unrelatedSignalIgnored,
            ControlDoubleDisposeWasIdempotent: true);
    }

    private string TransformWithControl(
        string requestIdentity,
        NativeControlHandle control,
        ulong maximumXsltInstructions)
    {
        lock (_gate)
        {
            ObjectDisposedException.ThrowIf(_engine.IsClosed, this);
            var request = Encoding.UTF8.GetBytes(requestIdentity);
            var outcome = NativeMethods.TransformWithControl(
                _engine.Value,
                request,
                (nuint)request.Length,
                control.Value,
                maximumXsltInstructions);
            return ReadTransformOutcome(outcome);
        }
    }

    private static string ReadTransformOutcome(ulong outcome)
    {
        var kind = NativeMethods.OutcomeKind(outcome);
        if (kind == 3)
        {
            throw ReadFailureAndRelease(outcome);
        }
        if (kind != 2)
        {
            NativeMethods.OutcomeRelease(outcome);
            throw new InvalidDataException("Native transform returned an invalid outcome.");
        }
        try
        {
            return Encoding.UTF8.GetString(ReadOutcome(outcome));
        }
        finally
        {
            NativeMethods.OutcomeRelease(outcome);
        }
    }

    public void Dispose() => _engine.Dispose();

    private static NativeFastXsltException ReadFailureAndRelease(ulong outcome)
    {
        try
        {
            var bytes = ReadOutcome(outcome);
            var offset = 0;
            var code = ReadField(bytes, ref offset);
            var category = ReadField(bytes, ref offset);
            var requestId = ReadField(bytes, ref offset);
            var resource = ReadField(bytes, ref offset);
            var start = ReadField(bytes, ref offset);
            var end = ReadField(bytes, ref offset);
            var detail = ReadField(bytes, ref offset);
            if (offset != bytes.Length)
            {
                throw new InvalidDataException("Native failure envelope has trailing bytes.");
            }
            return new NativeFastXsltException(
                code,
                category,
                requestId.Length == 0 ? null : requestId,
                resource.Length == 0
                    ? null
                    : new FastXsltDiagnosticLocation(
                        resource,
                        ulong.Parse(start),
                        ulong.Parse(end)),
                detail);
        }
        finally
        {
            NativeMethods.OutcomeRelease(outcome);
        }
    }

    private static byte[] ReadOutcome(ulong outcome)
    {
        var nativeLength = NativeMethods.OutcomeLength(outcome);
        if (nativeLength == nuint.MaxValue || nativeLength > 1_048_576)
        {
            throw new InvalidDataException("Native outcome length is invalid.");
        }
        var value = new byte[checked((int)nativeLength)];
        if (NativeMethods.OutcomeCopy(outcome, value, nativeLength) != 0)
        {
            throw new InvalidDataException("Native outcome copy failed.");
        }
        return value;
    }

    private static string ReadField(byte[] envelope, ref int offset)
    {
        if (envelope.Length - offset < sizeof(uint))
        {
            throw new InvalidDataException("Native failure envelope is truncated.");
        }
        var length = checked((int)BinaryPrimitives.ReadUInt32LittleEndian(
            envelope.AsSpan(offset, sizeof(uint))));
        offset += sizeof(uint);
        if (length > envelope.Length - offset)
        {
            throw new InvalidDataException("Native failure field is truncated.");
        }
        var value = Encoding.UTF8.GetString(envelope, offset, length);
        offset += length;
        return value;
    }

    private sealed class NativeEngineHandle : SafeHandleZeroOrMinusOneIsInvalid
    {
        public NativeEngineHandle(ulong value) : base(ownsHandle: true) =>
            SetHandle(unchecked((nint)value));

        public ulong Value => unchecked((ulong)handle);

        protected override bool ReleaseHandle() => NativeMethods.EngineRelease(Value) == 1;
    }

    private sealed class NativeControlHandle : SafeHandleZeroOrMinusOneIsInvalid
    {
        private NativeControlHandle(ulong value) : base(ownsHandle: true) =>
            SetHandle(unchecked((nint)value));

        public ulong Value => unchecked((ulong)handle);
        public bool FirstChargeObserved =>
            NativeMethods.ControlFirstChargeObserved(Value) == 1;

        public static NativeControlHandle Create(bool firstChargeBarrier)
        {
            var value = NativeMethods.ControlCreate(firstChargeBarrier ? 1u : 0u);
            if (value == 0)
            {
                throw new InvalidOperationException("Native invocation control creation failed.");
            }
            return new NativeControlHandle(value);
        }

        public void Cancel()
        {
            if (IsClosed || NativeMethods.ControlCancel(Value) != 1)
            {
                throw new ObjectDisposedException(nameof(NativeControlHandle));
            }
        }

        protected override bool ReleaseHandle() => NativeMethods.ControlRelease(Value) == 1;
    }

    private static class NativeMethods
    {
        private const string Library = "fastxslt_dotnet_workbench";

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_abi_version")]
        internal static extern uint AbiVersion();

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_create")]
        internal static extern ulong Create(
            byte[] sourceIdentity,
            nuint sourceIdentityLength,
            byte[] source,
            nuint sourceLength,
            byte[] stylesheetIdentity,
            nuint stylesheetIdentityLength,
            byte[] stylesheet,
            nuint stylesheetLength);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_transform")]
        internal static extern ulong Transform(
            ulong engineHandle,
            byte[] requestIdentity,
            nuint requestIdentityLength);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_transform_controlled")]
        internal static extern ulong TransformControlled(
            ulong engineHandle,
            byte[] requestIdentity,
            nuint requestIdentityLength,
            uint cancellationRequested,
            ulong maximumXsltInstructions);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_control_create")]
        internal static extern ulong ControlCreate(uint firstChargeBarrier);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_transform_with_control")]
        internal static extern ulong TransformWithControl(
            ulong engineHandle,
            byte[] requestIdentity,
            nuint requestIdentityLength,
            ulong controlHandle,
            ulong maximumXsltInstructions);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_control_cancel")]
        internal static extern uint ControlCancel(ulong controlHandle);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_control_first_charge_observed")]
        internal static extern uint ControlFirstChargeObserved(ulong controlHandle);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_control_release")]
        internal static extern uint ControlRelease(ulong controlHandle);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_outcome_kind")]
        internal static extern uint OutcomeKind(ulong outcomeHandle);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_outcome_length")]
        internal static extern nuint OutcomeLength(ulong outcomeHandle);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_outcome_copy")]
        internal static extern uint OutcomeCopy(
            ulong outcomeHandle,
            [Out] byte[] output,
            nuint outputCapacity);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_outcome_take_engine")]
        internal static extern ulong OutcomeTakeEngine(ulong outcomeHandle);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_outcome_release")]
        internal static extern uint OutcomeRelease(ulong outcomeHandle);

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_engine_release")]
        internal static extern uint EngineRelease(ulong engineHandle);
    }
}

public sealed record NativeActiveCancellationObservation(
    NativeFastXsltException Failure,
    double SignalToObservationMilliseconds,
    bool FirstChargeObserved,
    bool UnrelatedSignalIgnored,
    bool ControlDoubleDisposeWasIdempotent);

public sealed class NativeFastXsltException(
    string code,
    string category,
    string? requestId,
    FastXsltDiagnosticLocation? location,
    string detail) : Exception(detail)
{
    public string Code { get; } = code;
    public string Category { get; } = category;
    public string? RequestId { get; } = requestId;
    public FastXsltDiagnosticLocation? Location { get; } = location;
    public string Detail { get; } = detail;
}
