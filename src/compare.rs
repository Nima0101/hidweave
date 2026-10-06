use crate::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Category of a contract difference. All differences require review.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeKind {
    /// A report exists only in the new descriptor.
    ReportAdded,
    /// A report exists only in the old descriptor.
    ReportRemoved,
    /// Payload bit length or wire length changed.
    ReportLength,
    /// A field exists only in the new descriptor after matching.
    FieldAdded,
    /// A field exists only in the old descriptor after matching.
    FieldRemoved,
    /// A matched field changed one or more contract properties.
    FieldChanged,
}

/// A difference referencing fields in the original immutable layouts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// Report containing the difference.
    pub report: ReportKey,
    /// Difference category.
    pub kind: ChangeKind,
    /// Index in the old report's field list, when applicable.
    pub old_field: Option<usize>,
    /// Index in the new report's field list, when applicable.
    pub new_field: Option<usize>,
    /// Stable property names changed on a matched field.
    pub properties: Vec<&'static str>,
}

// Matching identities deliberately exclude offsets and bounds. A variable is identified
// by collection route, usage, then occurrence. Arrays by route then occurrence; padding
// by position. This explains moved axes without guessing unique identity for repeats.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Identity<'a> {
    route: &'a [Collection],
    category: u8,
    usage_or_offset: u32,
}
fn identity(field: &Field) -> Identity<'_> {
    let route = field
        .semantics
        .as_ref()
        .map_or(&[][..], |s| s.collections.as_slice());
    let (category, usage_or_offset) = match &field.meaning {
        Meaning::Padding => (0, field.bit_offset),
        Meaning::Variable(usage) => (1, *usage),
        Meaning::Array(_) => (2, 0),
    };
    Identity {
        route,
        category,
        usage_or_offset,
    }
}

fn properties(a: &Field, b: &Field) -> Vec<&'static str> {
    let mut changed = Vec::new();
    if a.bit_offset != b.bit_offset {
        changed.push("bit_offset");
    }
    if a.bit_size != b.bit_size {
        changed.push("bit_size");
    }
    if a.count != b.count {
        changed.push("count");
    }
    if a.meaning != b.meaning {
        changed.push("meaning");
    }
    match (&a.semantics, &b.semantics) {
        (Some(a), Some(b)) => {
            if a.flags != b.flags {
                changed.push("flags");
            }
            if (a.logical_min, a.logical_max) != (b.logical_min, b.logical_max) {
                changed.push("logical_range");
            }
            if (a.physical_min, a.physical_max) != (b.physical_min, b.physical_max) {
                changed.push("physical_range");
            }
            if (a.unit, a.unit_exponent) != (b.unit, b.unit_exponent) {
                changed.push("unit");
            }
            if a.collections != b.collections {
                changed.push("collections");
            }
        }
        (None, None) => {}
        _ => changed.push("semantics"),
    }
    changed
}

/// Compare supported normalized contracts. An empty result means equality under
/// contract version 1, not universal HID/OS compatibility. Source offsets do not
/// affect equality. Matching uses collection/usage/occurrence before position.
pub fn compare(old: &Layout, new: &Layout) -> Vec<Change> {
    let keys: BTreeSet<_> = old
        .reports
        .keys()
        .chain(new.reports.keys())
        .copied()
        .collect();
    let mut changes = Vec::new();
    for key in keys {
        let mut add = |kind, old_field, new_field, properties| {
            changes.push(Change {
                report: key,
                kind,
                old_field,
                new_field,
                properties,
            })
        };
        let (a, b) = match (old.reports.get(&key), new.reports.get(&key)) {
            (None, Some(_)) => {
                add(ChangeKind::ReportAdded, None, None, vec![]);
                continue;
            }
            (Some(_), None) => {
                add(ChangeKind::ReportRemoved, None, None, vec![]);
                continue;
            }
            (Some(a), Some(b)) => (a, b),
            _ => unreachable!("union of report keys"),
        };
        if a.payload_bits != b.payload_bits {
            add(
                ChangeKind::ReportLength,
                None,
                None,
                if a.wire_bytes() != b.wire_bytes() {
                    vec!["payload_bits", "wire_bytes"]
                } else {
                    vec!["payload_bits"]
                },
            );
        }
        let mut identities = BTreeMap::<_, VecDeque<usize>>::new();
        for (index, field) in b.fields.iter().enumerate() {
            identities
                .entry(identity(field))
                .or_default()
                .push_back(index);
        }
        let mut matches = vec![None; a.fields.len()];
        let mut used = vec![false; b.fields.len()];
        for (i, field) in a.fields.iter().enumerate() {
            if let Some(j) = identities
                .get_mut(&identity(field))
                .and_then(VecDeque::pop_front)
            {
                matches[i] = Some(j);
                used[j] = true;
            }
        }
        let mut positions = BTreeMap::new();
        for (j, field) in b.fields.iter().enumerate() {
            if !used[j] {
                positions.insert(field.bit_offset, j);
            }
        }
        for (i, field) in a.fields.iter().enumerate() {
            if matches[i].is_none() {
                if let Some(j) = positions.remove(&field.bit_offset) {
                    matches[i] = Some(j);
                    used[j] = true;
                }
            }
            match matches[i] {
                Some(j) if !field.contract_eq(&b.fields[j]) => add(
                    ChangeKind::FieldChanged,
                    Some(i),
                    Some(j),
                    properties(field, &b.fields[j]),
                ),
                None => add(ChangeKind::FieldRemoved, Some(i), None, vec![]),
                _ => {}
            }
        }
        for (j, is_used) in used.iter().enumerate() {
            if !is_used {
                add(ChangeKind::FieldAdded, None, Some(j), vec![]);
            }
        }
    }
    changes
}
