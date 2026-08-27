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
        if (NativeMethods.AbiVersion() != 0)
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
            var detail = ReadField(bytes, ref offset);
            if (offset != bytes.Length)
            {
                throw new InvalidDataException("Native failure envelope has trailing bytes.");
            }
            return new NativeFastXsltException(
                code,
                category,
                requestId.Length == 0 ? null : requestId,
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

public sealed class NativeFastXsltException(
    string code,
    string category,
    string? requestId,
    string detail) : Exception(detail)
{
    public string Code { get; } = code;
    public string Category { get; } = category;
    public string? RequestId { get; } = requestId;
    public string Detail { get; } = detail;
}
