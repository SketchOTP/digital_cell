use crate::{EventId, LineageId, OrganismId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AncestryRecord {
    pub organism_id: OrganismId,
    pub parent_id: Option<OrganismId>,
    pub lineage_id: LineageId,
    pub generation: u32,
    pub birth_event_id: EventId,
    pub birth_time: f64,
    pub death_time: Option<f64>,
    pub phenotype_history: Vec<String>,
    pub hereditary_state_history: Vec<String>,
}

#[derive(Debug, Error, PartialEq)]
pub enum LineageError {
    #[error("organism {0} already has ancestry")]
    Duplicate(OrganismId),
    #[error("parent {0} has no ancestry")]
    MissingParent(OrganismId),
    #[error("organism {0} has an invalid generation")]
    InvalidGeneration(OrganismId),
    #[error("organism {0} has an invalid birth event")]
    InvalidBirthEvent(OrganismId),
    #[error("organism {0} has more than one death")]
    DoubleDeath(OrganismId),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LineageTracker {
    pub ancestry: BTreeMap<OrganismId, AncestryRecord>,
}

impl LineageTracker {
    pub fn register_founder(&mut self, organism_id: OrganismId, lineage_id: LineageId, birth_event_id: EventId, birth_time: f64) -> Result<(), LineageError> {
        if self.ancestry.contains_key(&organism_id) { return Err(LineageError::Duplicate(organism_id)); }
        if birth_event_id == 0 { return Err(LineageError::InvalidBirthEvent(organism_id)); }
        self.ancestry.insert(organism_id, AncestryRecord {
            organism_id, parent_id: None, lineage_id, generation: 0, birth_event_id, birth_time,
            death_time: None, phenotype_history: Vec::new(), hereditary_state_history: Vec::new(),
        });
        Ok(())
    }

    pub fn register_offspring(&mut self, organism_id: OrganismId, parent_id: OrganismId, lineage_id: LineageId, birth_event_id: EventId, birth_time: f64) -> Result<(), LineageError> {
        if self.ancestry.contains_key(&organism_id) { return Err(LineageError::Duplicate(organism_id)); }
        let parent = self.ancestry.get(&parent_id).ok_or(LineageError::MissingParent(parent_id))?;
        if birth_event_id == 0 { return Err(LineageError::InvalidBirthEvent(organism_id)); }
        self.ancestry.insert(organism_id, AncestryRecord {
            organism_id, parent_id: Some(parent_id), lineage_id, generation: parent.generation + 1,
            birth_event_id, birth_time, death_time: None, phenotype_history: Vec::new(), hereditary_state_history: Vec::new(),
        });
        Ok(())
    }

    pub fn record_observation(&mut self, organism_id: OrganismId, phenotype: String, hereditary_state: String) -> Result<(), LineageError> {
        let record = self.ancestry.get_mut(&organism_id).ok_or(LineageError::MissingParent(organism_id))?;
        record.phenotype_history.push(phenotype);
        record.hereditary_state_history.push(hereditary_state);
        Ok(())
    }

    pub fn record_death(&mut self, organism_id: OrganismId, time: f64) -> Result<(), LineageError> {
        let record = self.ancestry.get_mut(&organism_id).ok_or(LineageError::MissingParent(organism_id))?;
        if record.death_time.is_some() { return Err(LineageError::DoubleDeath(organism_id)); }
        record.death_time = Some(time);
        Ok(())
    }

    pub fn parent(&self, organism_id: OrganismId) -> Option<OrganismId> { self.ancestry.get(&organism_id).and_then(|r| r.parent_id) }
    pub fn children(&self, parent_id: OrganismId) -> Vec<OrganismId> { self.ancestry.values().filter(|r| r.parent_id == Some(parent_id)).map(|r| r.organism_id).collect() }
    pub fn lineage_members(&self, lineage_id: LineageId) -> Vec<OrganismId> { self.ancestry.values().filter(|r| r.lineage_id == lineage_id).map(|r| r.organism_id).collect() }
    pub fn generation(&self, organism_id: OrganismId) -> Option<u32> { self.ancestry.get(&organism_id).map(|r| r.generation) }

    /// Maximum descendant depth below `organism_id`; a leaf has depth zero.
    pub fn descendant_depth(&self, organism_id: OrganismId) -> u32 {
        self.children(organism_id).into_iter().map(|child| 1 + self.descendant_depth(child)).max().unwrap_or(0)
    }

    /// Compatibility alias retained only with unambiguous descendant semantics.
    pub fn lineage_depth(&self, organism_id: OrganismId) -> u32 { self.descendant_depth(organism_id) }

    pub fn ancestor_depth(&self, organism_id: OrganismId) -> u32 {
        let mut depth = 0;
        let mut cursor = self.parent(organism_id);
        while let Some(parent) = cursor { depth += 1; cursor = self.parent(parent); }
        depth
    }

    pub fn descendant_count(&self, organism_id: OrganismId) -> usize {
        let mut seen = BTreeSet::new();
        let mut frontier = self.children(organism_id);
        while let Some(child) = frontier.pop() { if seen.insert(child) { frontier.extend(self.children(child)); } }
        seen.len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GenerationTracker {
    pub max_generation: u32,
    pub completed_births: u64,
    pub completed_fissions: u64,
    pub generation_distribution: BTreeMap<u32, u64>,
    pub lineage_depth: u32,
    pub time_to_first_birth: Option<f64>,
    pub generation_times: Vec<f64>,
    pub founder_time: Option<f64>,
    pub last_fission_time: Option<f64>,
}

impl GenerationTracker {
    pub fn record_founder(&mut self, time: f64) { self.founder_time = Some(time); }

    pub fn record_completed_fission(&mut self, generation: u32, time: f64) {
        self.completed_fissions += 1;
        self.completed_births += 2;
        self.max_generation = self.max_generation.max(generation);
        *self.generation_distribution.entry(generation).or_default() += 2;
        self.time_to_first_birth.get_or_insert_with(|| time - self.founder_time.unwrap_or(time));
        let prior = self.last_fission_time.or(self.founder_time);
        if let Some(prior) = prior { self.generation_times.push((time - prior).max(0.0)); }
        self.last_fission_time = Some(time);
    }

    pub fn median_generation_time(&self) -> Option<f64> {
        if self.generation_times.is_empty() { return None; }
        let mut values = self.generation_times.clone();
        values.sort_by(|a, b| a.total_cmp(b));
        Some(values[(values.len() - 1) / 2])
    }
}
