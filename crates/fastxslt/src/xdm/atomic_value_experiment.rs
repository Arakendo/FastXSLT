//! Private XDM atomic-value identity retained independently of `XPath` operations.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuiltinAtomicType {
    String,
    UntypedAtomic,
    Boolean,
    Integer,
    Decimal,
    Float,
    Double,
    Duration,
    DayTimeDuration,
    YearMonthDuration,
    DateTime,
    Date,
    Time,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AtomicValue {
    atomic_type: BuiltinAtomicType,
    lexical: String,
}

impl AtomicValue {
    pub(crate) fn string(value: impl Into<String>) -> Self {
        Self::from_validated_lexical(BuiltinAtomicType::String, value)
    }

    pub(crate) fn untyped(value: impl Into<String>) -> Self {
        Self::from_validated_lexical(BuiltinAtomicType::UntypedAtomic, value)
    }

    pub(crate) fn from_validated_lexical(
        atomic_type: BuiltinAtomicType,
        lexical: impl Into<String>,
    ) -> Self {
        Self {
            atomic_type,
            lexical: lexical.into(),
        }
    }

    pub(crate) fn atomic_type(&self) -> BuiltinAtomicType {
        self.atomic_type
    }

    pub(crate) fn lexical(&self) -> &str {
        &self.lexical
    }
}
