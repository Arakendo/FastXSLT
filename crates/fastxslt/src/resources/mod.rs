//! Private M1 experiment for bounded, memory-owned resource admission.
//!
//! This module is test-only until AR-0003 has enough evidence to settle public
//! identity, lifetime, and batch contracts.

use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ResourceLimits {
    entries: usize,
    entry_bytes: usize,
    total_bytes: usize,
}

impl ResourceLimits {
    pub(crate) const fn new(
        max_entries: usize,
        max_entry_bytes: usize,
        max_total_bytes: usize,
    ) -> Self {
        Self {
            entries: max_entries,
            entry_bytes: max_entry_bytes,
            total_bytes: max_total_bytes,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum AdmissionError {
    EmptyIdentity,
    DuplicateIdentity {
        identity: String,
    },
    EntryLimit {
        maximum: usize,
    },
    EntryTooLarge {
        identity: String,
        actual: usize,
        maximum: usize,
    },
    AggregateTooLarge {
        attempted: usize,
        maximum: usize,
    },
    AggregateSizeOverflow,
}

#[derive(Debug)]
pub(crate) struct ResourceSetBuilder {
    limits: ResourceLimits,
    entries: BTreeMap<String, Arc<[u8]>>,
    total_bytes: usize,
}

impl ResourceSetBuilder {
    pub(crate) fn new(limits: ResourceLimits) -> Self {
        Self {
            limits,
            entries: BTreeMap::new(),
            total_bytes: 0,
        }
    }

    pub(crate) fn admit(
        &mut self,
        identity: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Result<(), AdmissionError> {
        let identity = identity.into();
        if identity.is_empty() {
            return Err(AdmissionError::EmptyIdentity);
        }

        if self.entries.contains_key(&identity) {
            return Err(AdmissionError::DuplicateIdentity { identity });
        }

        if self.entries.len() >= self.limits.entries {
            return Err(AdmissionError::EntryLimit {
                maximum: self.limits.entries,
            });
        }

        if bytes.len() > self.limits.entry_bytes {
            return Err(AdmissionError::EntryTooLarge {
                identity,
                actual: bytes.len(),
                maximum: self.limits.entry_bytes,
            });
        }

        let attempted = self
            .total_bytes
            .checked_add(bytes.len())
            .ok_or(AdmissionError::AggregateSizeOverflow)?;
        if attempted > self.limits.total_bytes {
            return Err(AdmissionError::AggregateTooLarge {
                attempted,
                maximum: self.limits.total_bytes,
            });
        }

        self.entries.insert(identity, Arc::from(bytes));
        self.total_bytes = attempted;
        Ok(())
    }

    pub(crate) fn seal(self) -> ResourceSnapshot {
        ResourceSnapshot {
            entries: Arc::new(self.entries),
            total_bytes: self.total_bytes,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResourceSnapshot {
    entries: Arc<BTreeMap<String, Arc<[u8]>>>,
    total_bytes: usize,
}

impl ResourceSnapshot {
    pub(crate) fn get(&self, identity: &str) -> Option<&[u8]> {
        self.entries.get(identity).map(AsRef::as_ref)
    }

    pub(crate) fn same_generation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.entries, &other.entries)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn total_bytes(&self) -> usize {
        self.total_bytes
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{AdmissionError, ResourceLimits, ResourceSetBuilder};

    static NEXT_TEMP_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TempDirectory(PathBuf);

    impl TempDirectory {
        fn new() -> Self {
            let sequence = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "fastxslt-resource-test-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create isolated test directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("remove isolated test directory");
        }
    }

    fn generous_limits() -> ResourceLimits {
        ResourceLimits::new(8, 4096, 8192)
    }

    #[test]
    fn sealed_snapshot_preserves_distinct_logical_identities_for_equal_bytes() {
        let mut builder = ResourceSetBuilder::new(generous_limits());
        builder
            .admit("urn:fastxslt:test:first", b"same".to_vec())
            .expect("admit first resource");
        builder
            .admit("urn:fastxslt:test:second", b"same".to_vec())
            .expect("admit second resource");

        let snapshot = builder.seal();

        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot.total_bytes(), 8);
        assert_eq!(snapshot.get("urn:fastxslt:test:first"), Some(&b"same"[..]));
        assert_eq!(snapshot.get("urn:fastxslt:test:second"), Some(&b"same"[..]));
    }

    #[test]
    fn duplicate_identity_is_rejected_without_changing_retained_bytes() {
        let mut builder = ResourceSetBuilder::new(generous_limits());
        builder
            .admit("urn:fastxslt:test:source", b"first".to_vec())
            .expect("admit original resource");

        assert_eq!(
            builder.admit("urn:fastxslt:test:source", b"replacement".to_vec()),
            Err(AdmissionError::DuplicateIdentity {
                identity: "urn:fastxslt:test:source".to_owned(),
            })
        );

        let snapshot = builder.seal();
        assert_eq!(snapshot.total_bytes(), 5);
        assert_eq!(
            snapshot.get("urn:fastxslt:test:source"),
            Some(&b"first"[..])
        );
    }

    #[test]
    fn explicit_entry_and_aggregate_limits_are_enforced_before_mutation() {
        let mut entry_limited = ResourceSetBuilder::new(ResourceLimits::new(1, 4, 4));
        entry_limited
            .admit("one", b"1234".to_vec())
            .expect("admit boundary-sized resource");
        assert_eq!(
            entry_limited.admit("two", Vec::new()),
            Err(AdmissionError::EntryLimit { maximum: 1 })
        );

        let mut byte_limited = ResourceSetBuilder::new(ResourceLimits::new(2, 3, 5));
        assert_eq!(
            byte_limited.admit("large", b"1234".to_vec()),
            Err(AdmissionError::EntryTooLarge {
                identity: "large".to_owned(),
                actual: 4,
                maximum: 3,
            })
        );
        byte_limited
            .admit("first", b"123".to_vec())
            .expect("admit first aggregate resource");
        assert_eq!(
            byte_limited.admit("second", b"456".to_vec()),
            Err(AdmissionError::AggregateTooLarge {
                attempted: 6,
                maximum: 5,
            })
        );

        let snapshot = byte_limited.seal();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot.total_bytes(), 3);
        assert_eq!(snapshot.get("second"), None);
    }

    #[test]
    fn empty_identity_is_rejected() {
        let mut builder = ResourceSetBuilder::new(generous_limits());
        assert_eq!(
            builder.admit(String::new(), Vec::new()),
            Err(AdmissionError::EmptyIdentity)
        );
    }

    #[test]
    fn imported_golden_files_can_be_renamed_and_removed_after_admission() {
        let temporary = TempDirectory::new();
        let source_bytes = include_bytes!("../../../../corpus/golden/hello/input.xml");
        let stylesheet_bytes = include_bytes!("../../../../corpus/golden/hello/stylesheet.xsl");
        let resources: [(&str, &str, &[u8]); 2] = [
            (
                "source.xml",
                "urn:fastxslt:golden:hello:source",
                source_bytes,
            ),
            (
                "stylesheet.xsl",
                "urn:fastxslt:golden:hello:stylesheet",
                stylesheet_bytes,
            ),
        ];

        let mut builder = ResourceSetBuilder::new(generous_limits());
        for (file_name, identity, expected_bytes) in resources {
            let import_path = temporary.path().join(file_name);
            let renamed_path = import_path.with_extension("imported");
            fs::write(&import_path, expected_bytes).expect("write temporary import file");

            let imported = fs::read(&import_path).expect("import and close file");
            builder
                .admit(identity, imported)
                .expect("admit imported bytes");

            fs::rename(&import_path, &renamed_path)
                .expect("rename file after import while builder remains alive");
            fs::remove_file(&renamed_path)
                .expect("remove renamed file while builder remains alive");
        }

        let snapshot = builder.seal();

        assert_eq!(snapshot.len(), 2);
        assert_eq!(
            snapshot.get("urn:fastxslt:golden:hello:source"),
            Some(&source_bytes[..])
        );
        assert_eq!(
            snapshot.get("urn:fastxslt:golden:hello:stylesheet"),
            Some(&stylesheet_bytes[..])
        );
    }
}
