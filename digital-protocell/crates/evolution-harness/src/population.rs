use crate::{EventId, LineageId, OrganismId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PopulationState {
    Founder,
    Living,
    NewOffspring,
    Dead,
    Removed,
    ExtinctLineage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PopulationRecord {
    pub organism_id: OrganismId,
    pub parent_id: Option<OrganismId>,
    pub lineage_id: LineageId,
    pub birth_event_id: EventId,
    pub birth_time: f64,
    pub birth_generation: u32,
    pub placement: [f64; 2],
    pub death_time: Option<f64>,
    pub state: PopulationState,
}

#[derive(Debug, Error, PartialEq)]
pub enum PopulationError {
    #[error("organism {0} already exists")]
    Duplicate(OrganismId),
    #[error("organism {0} does not exist")]
    Missing(OrganismId),
    #[error("organism {0} is not living")]
    NotLiving(OrganismId),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PopulationManager {
    pub records: BTreeMap<OrganismId, PopulationRecord>,
    pub next_organism_id: OrganismId,
}

impl PopulationManager {
    pub fn create_founder(
        &mut self,
        organism_id: OrganismId,
        lineage_id: LineageId,
        birth_event_id: EventId,
        birth_time: f64,
        placement: [f64; 2],
    ) -> Result<(), PopulationError> {
        self.insert(PopulationRecord {
            organism_id,
            parent_id: None,
            lineage_id,
            birth_event_id,
            birth_time,
            birth_generation: 0,
            placement,
            death_time: None,
            state: PopulationState::Founder,
        })
    }

    pub fn register_offspring(
        &mut self,
        organism_id: OrganismId,
        parent_id: OrganismId,
        lineage_id: LineageId,
        birth_event_id: EventId,
        birth_time: f64,
        birth_generation: u32,
        placement: [f64; 2],
    ) -> Result<(), PopulationError> {
        self.insert(PopulationRecord {
            organism_id,
            parent_id: Some(parent_id),
            lineage_id,
            birth_event_id,
            birth_time,
            birth_generation,
            placement,
            death_time: None,
            state: PopulationState::NewOffspring,
        })
    }

    fn insert(&mut self, record: PopulationRecord) -> Result<(), PopulationError> {
        if self.records.contains_key(&record.organism_id) {
            return Err(PopulationError::Duplicate(record.organism_id));
        }
        self.next_organism_id = self.next_organism_id.max(record.organism_id + 1);
        self.records.insert(record.organism_id, record);
        Ok(())
    }

    pub fn mark_living(&mut self, organism_id: OrganismId) -> Result<(), PopulationError> {
        let record = self
            .records
            .get_mut(&organism_id)
            .ok_or(PopulationError::Missing(organism_id))?;
        if matches!(
            record.state,
            PopulationState::Dead | PopulationState::Removed
        ) {
            return Err(PopulationError::NotLiving(organism_id));
        }
        record.state = PopulationState::Living;
        Ok(())
    }

    pub fn mark_dead(&mut self, organism_id: OrganismId, time: f64) -> Result<(), PopulationError> {
        let record = self
            .records
            .get_mut(&organism_id)
            .ok_or(PopulationError::Missing(organism_id))?;
        if matches!(
            record.state,
            PopulationState::Dead | PopulationState::Removed
        ) {
            return Err(PopulationError::NotLiving(organism_id));
        }
        record.state = PopulationState::Dead;
        record.death_time = Some(time);
        Ok(())
    }

    pub fn living_ids(&self) -> Vec<OrganismId> {
        self.records
            .values()
            .filter(|record| {
                matches!(
                    record.state,
                    PopulationState::Founder
                        | PopulationState::Living
                        | PopulationState::NewOffspring
                )
            })
            .map(|record| record.organism_id)
            .collect()
    }

    pub fn living_count(&self) -> usize {
        self.living_ids().len()
    }

    pub fn get(&self, organism_id: OrganismId) -> Option<&PopulationRecord> {
        self.records.get(&organism_id)
    }
}
