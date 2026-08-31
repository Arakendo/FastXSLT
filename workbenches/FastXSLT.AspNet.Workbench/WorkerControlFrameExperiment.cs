using System.Buffers.Binary;
using System.Text;

public static class WorkerControlFrameExperiment
{
    private const byte Cancel = 7;
    private const int MaximumFrameBytes = 1_048_576;

    public static async Task<object> ExerciseAsync(int pairs = 10_000)
    {
        using var output = new ByteFragmentingCaptureStream();
        var writer = new SerializedWorkerControlWriter(output, MaximumFrameBytes);
        var expected = new HashSet<string>(StringComparer.Ordinal);
        var sends = new List<Task>(pairs * 2);
        for (var index = 0; index < pairs; index++)
        {
            var first = $"cancel-a-{index:D5}";
            var second = $"cancel-b-{index:D5}";
            expected.Add(first);
            expected.Add(second);
            sends.Add(writer.WriteCancellationAsync(Cancel, first));
            sends.Add(writer.WriteCancellationAsync(Cancel, second));
        }

        await Task.WhenAll(sends);
        var captured = output.ToArray();
        var observed = ParseCancellationFrames(captured);
        var distinct = observed.ToHashSet(StringComparer.Ordinal);
        var framesIntact = observed.Count == expected.Count &&
            distinct.Count == expected.Count &&
            distinct.SetEquals(expected);
        return new
        {
            pairs,
            framesExpected = expected.Count,
            framesObserved = observed.Count,
            capturedBytes = captured.Length,
            framesIntact,
            writesWereFragmentedAfterEveryByte = true,
            outboundControlFramesSerialized = true
        };
    }

    private static List<string> ParseCancellationFrames(byte[] bytes)
    {
        var identities = new List<string>();
        var offset = 0;
        while (offset < bytes.Length)
        {
            if (bytes[offset++] != Cancel || bytes.Length - offset < sizeof(int))
            {
                throw new InvalidDataException("Captured control frame has an invalid opcode or length.");
            }
            var length = BinaryPrimitives.ReadInt32LittleEndian(
                bytes.AsSpan(offset, sizeof(int)));
            offset += sizeof(int);
            if (length < 0 || length > MaximumFrameBytes || bytes.Length - offset < length)
            {
                throw new InvalidDataException("Captured control frame has an invalid payload length.");
            }
            identities.Add(Encoding.UTF8.GetString(bytes, offset, length));
            offset += length;
        }
        return identities;
    }

    private sealed class ByteFragmentingCaptureStream : Stream
    {
        private readonly MemoryStream _capture = new();

        public byte[] ToArray() => _capture.ToArray();

        public override bool CanRead => false;
        public override bool CanSeek => false;
        public override bool CanWrite => true;
        public override long Length => _capture.Length;
        public override long Position
        {
            get => _capture.Position;
            set => throw new NotSupportedException();
        }

        public override void Flush() => _capture.Flush();

        public override Task FlushAsync(CancellationToken cancellationToken) =>
            _capture.FlushAsync(cancellationToken);

        public override async ValueTask WriteAsync(
            ReadOnlyMemory<byte> buffer,
            CancellationToken cancellationToken = default)
        {
            for (var index = 0; index < buffer.Length; index++)
            {
                cancellationToken.ThrowIfCancellationRequested();
                var value = buffer.Span[index];
                _capture.WriteByte(value);
                await Task.Yield();
            }
        }

        public override void Write(byte[] buffer, int offset, int count) =>
            _capture.Write(buffer, offset, count);

        public override int Read(byte[] buffer, int offset, int count) =>
            throw new NotSupportedException();

        public override long Seek(long offset, SeekOrigin origin) =>
            throw new NotSupportedException();

        public override void SetLength(long value) => throw new NotSupportedException();

        protected override void Dispose(bool disposing)
        {
            if (disposing)
            {
                _capture.Dispose();
            }
            base.Dispose(disposing);
        }
    }
}
