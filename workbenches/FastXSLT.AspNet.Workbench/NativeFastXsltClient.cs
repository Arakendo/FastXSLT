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
        AssertAbiVersion();
        var sourceIdentityBytes = Encoding.UTF8.GetBytes(sourceIdentity);
        var stylesheetIdentityBytes = Encoding.UTF8.GetBytes(stylesheetIdentity);
        return FromCreationOutcome(NativeMethods.Create(
            sourceIdentityBytes,
            (nuint)sourceIdentityBytes.Length,
            source,
            (nuint)source.Length,
            stylesheetIdentityBytes,
            (nuint)stylesheetIdentityBytes.Length,
            stylesheet,
            (nuint)stylesheet.Length));
    }

    public static NativeFastXsltClient CreateWithStylesheetDependency(
        string sourceIdentity,
        byte[] source,
        string stylesheetIdentity,
        byte[] stylesheet,
        string dependencyIdentity,
        byte[] dependency,
        bool admitted,
        bool denied)
    {
        if (!admitted && dependency.Length != 0)
        {
            throw new ArgumentException(
                "An unadmitted stylesheet dependency must not carry bytes.",
                nameof(dependency));
        }
        AssertAbiVersion();
        var sourceIdentityBytes = Encoding.UTF8.GetBytes(sourceIdentity);
        var stylesheetIdentityBytes = Encoding.UTF8.GetBytes(stylesheetIdentity);
        var dependencyIdentityBytes = Encoding.UTF8.GetBytes(dependencyIdentity);
        return FromCreationOutcome(NativeMethods.CreateWithStylesheetDependency(
            sourceIdentityBytes,
            (nuint)sourceIdentityBytes.Length,
            source,
            (nuint)source.Length,
            stylesheetIdentityBytes,
            (nuint)stylesheetIdentityBytes.Length,
            stylesheet,
            (nuint)stylesheet.Length,
            dependencyIdentityBytes,
            (nuint)dependencyIdentityBytes.Length,
            dependency,
            (nuint)dependency.Length,
            admitted ? 1u : 0u,
            denied ? 1u : 0u));
    }

    private static void AssertAbiVersion()
    {
        if (NativeMethods.AbiVersion() != 2)
        {
            throw new InvalidOperationException("Unexpected native FastXSLT workbench ABI version.");
        }
    }

    public static NativeRegistryObservation ObserveRegistry()
    {
        var engines = NativeMethods.RegistryEngineCount();
        var controls = NativeMethods.RegistryControlCount();
        var outcomes = NativeMethods.RegistryOutcomeCount();
        var outcomePayloadBytes = NativeMethods.RegistryOutcomePayloadBytes();
        if (engines == nuint.MaxValue ||
            controls == nuint.MaxValue ||
            outcomes == nuint.MaxValue ||
            outcomePayloadBytes == nuint.MaxValue)
        {
            throw new InvalidOperationException("Native registry observation failed.");
        }
        return new NativeRegistryObservation(
            checked((ulong)engines),
            checked((ulong)controls),
            checked((ulong)outcomes),
            checked((ulong)outcomePayloadBytes));
    }

    private static NativeFastXsltClient FromCreationOutcome(ulong outcome)
    {
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

    public NativeRetainedOutcome TransformRetained(string requestIdentity)
    {
        lock (_gate)
        {
            ObjectDisposedException.ThrowIf(_engine.IsClosed, this);
            var request = Encoding.UTF8.GetBytes(requestIdentity);
            var outcome = NativeMethods.Transform(_engine.Value, request, (nuint)request.Length);
            if (outcome == 0)
            {
                throw new InvalidOperationException("Native transform could not retain its outcome.");
            }
            return new NativeRetainedOutcome(outcome);
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

    internal NativeActiveTransform StartBarrierTransform(string requestIdentity)
    {
        ObjectDisposedException.ThrowIf(_engine.IsClosed, this);
        var control = NativeControlHandle.Create(firstChargeBarrier: true);
        try
        {
            return new NativeActiveTransform(
                control,
                Task.Run(() => TransformWithControl(
                    requestIdentity,
                    control,
                    maximumXsltInstructions: 1_000_000)));
        }
        catch
        {
            control.Dispose();
            throw;
        }
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
            return DecodeFailure(ReadOutcome(outcome));
        }
        finally
        {
            NativeMethods.OutcomeRelease(outcome);
        }
    }

    private static NativeFastXsltException DecodeFailure(byte[] bytes)
    {
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

    public sealed class NativeRetainedOutcome : IDisposable
    {
        private readonly NativeOutcomeHandle _outcome;

        internal NativeRetainedOutcome(ulong value) => _outcome = new NativeOutcomeHandle(value);

        public string ReadResult()
        {
            ObjectDisposedException.ThrowIf(_outcome.IsClosed, this);
            if (NativeMethods.OutcomeKind(_outcome.Value) != 2)
            {
                throw new InvalidDataException("Retained native outcome is not a result.");
            }
            return Encoding.UTF8.GetString(ReadOutcome(_outcome.Value));
        }

        public NativeFastXsltException ReadFailure()
        {
            ObjectDisposedException.ThrowIf(_outcome.IsClosed, this);
            if (NativeMethods.OutcomeKind(_outcome.Value) != 3)
            {
                throw new InvalidDataException("Retained native outcome is not a failure.");
            }
            return DecodeFailure(ReadOutcome(_outcome.Value));
        }

        public void Dispose() => _outcome.Dispose();
    }

    private sealed class NativeOutcomeHandle : SafeHandleZeroOrMinusOneIsInvalid
    {
        public NativeOutcomeHandle(ulong value) : base(ownsHandle: true) =>
            SetHandle(unchecked((nint)value));

        public ulong Value => unchecked((ulong)handle);

        protected override bool ReleaseHandle() => NativeMethods.OutcomeRelease(Value) == 1;
    }

    internal sealed class NativeControlHandle : SafeHandleZeroOrMinusOneIsInvalid
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

    internal sealed class NativeActiveTransform(
        NativeControlHandle control,
        Task<string> completion) : IDisposable
    {
        public bool FirstChargeObserved => control.FirstChargeObserved;
        public bool IsCompleted => completion.IsCompleted;

        public void Cancel() => control.Cancel();

        public async Task<NativeFastXsltException> ObserveCancellationAsync()
        {
            try
            {
                _ = await completion;
                throw new InvalidOperationException(
                    "Barrier-controlled native transform unexpectedly completed.");
            }
            catch (NativeFastXsltException failure)
            {
                return failure;
            }
        }

        public async Task AwaitCompletionIgnoringFailureAsync()
        {
            try
            {
                _ = await completion;
            }
            catch
            {
                // Cleanup waits for native use of the numeric control to end;
                // the caller validates the operation failure separately.
            }
        }

        public void Dispose() => control.Dispose();
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

        [DllImport(
            Library,
            EntryPoint = "fastxslt_workbench_v0_create_with_stylesheet_dependency")]
        internal static extern ulong CreateWithStylesheetDependency(
            byte[] sourceIdentity,
            nuint sourceIdentityLength,
            byte[] source,
            nuint sourceLength,
            byte[] stylesheetIdentity,
            nuint stylesheetIdentityLength,
            byte[] stylesheet,
            nuint stylesheetLength,
            byte[] dependencyIdentity,
            nuint dependencyIdentityLength,
            byte[] dependency,
            nuint dependencyLength,
            uint admitDependency,
            uint denyDependency);

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

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_registry_engine_count")]
        internal static extern nuint RegistryEngineCount();

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_registry_control_count")]
        internal static extern nuint RegistryControlCount();

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_registry_outcome_count")]
        internal static extern nuint RegistryOutcomeCount();

        [DllImport(Library, EntryPoint = "fastxslt_workbench_v0_registry_outcome_payload_bytes")]
        internal static extern nuint RegistryOutcomePayloadBytes();
    }
}

public sealed record NativeRegistryObservation(
    ulong EngineHandles,
    ulong ControlHandles,
    ulong OutcomeHandles,
    ulong OutcomePayloadBytes);

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
