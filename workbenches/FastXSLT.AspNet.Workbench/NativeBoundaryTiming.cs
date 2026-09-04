internal sealed record NativeBoundaryTransformSample(
    string Result,
    NativeBoundaryTiming Timing);

internal sealed record NativeBoundaryTiming(
    double GateMicroseconds,
    double RequestEncodingMicroseconds,
    double TransformExportMicroseconds,
    double OutcomeKindMicroseconds,
    double OutcomeLengthMicroseconds,
    double BufferAllocationMicroseconds,
    double OutcomeCopyMicroseconds,
    double ResultDecodingMicroseconds,
    double OutcomeReleaseMicroseconds,
    double InstrumentedTotalMicroseconds);
