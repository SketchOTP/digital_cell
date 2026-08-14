use crate::{stable_hash, ProtocolError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub type EventId = u64;
pub type OrganismId = u64;
pub type LineageId = u64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventType {
    #[serde(rename = "FOUNDER_CREATED")]
    FounderCreated,
    #[serde(rename = "BIRTH")]
    Birth,
    #[serde(rename = "FISSION_STARTED")]
    FissionStarted,
    #[serde(rename = "FISSION_COMPLETED")]
    FissionCompleted,
    #[serde(rename = "DEATH")]
    Death,
    #[serde(rename = "EXTINCTION")]
    Extinction,
    #[serde(rename = "RESOURCE_PULSE")]
    ResourcePulse,
    #[serde(rename = "SCARCITY_STARTED")]
    ScarcityStarted,
    #[serde(rename = "SCARCITY_ENDED")]
    ScarcityEnded,
    #[serde(rename = "DAMAGE_APPLIED")]
    DamageApplied,
    #[serde(rename = "ENVIRONMENT_SWITCH")]
    EnvironmentSwitch,
    #[serde(rename = "MUTATION")]
    Mutation,
    #[serde(rename = "TRANSFER")]
    Transfer,
    #[serde(rename = "EXPERIMENT_END")]
    ExperimentEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventV1 {
    pub schema: String,
    pub event_id: EventId,
    pub accepted_simulated_time: u64,
    pub accepted_step: u64,
    pub replicate: u32,
    pub event_type: EventType,
    pub organism_id: Option<OrganismId>,
    pub parent_id: Option<OrganismId>,
    pub lineage_id: Option<LineageId>,
    pub environment_id: String,
    pub protocol_id: String,
    pub metadata: BTreeMap<String, String>,
}

impl EventV1 {
    fn base(
        event_id: EventId,
        accepted_simulated_time: u64,
        accepted_step: u64,
        replicate: u32,
        event_type: EventType,
        environment_id: &str,
        protocol_id: &str,
    ) -> Self {
        Self {
            schema: "EventV1".into(),
            event_id,
            accepted_simulated_time,
            accepted_step,
            replicate,
            event_type,
            organism_id: None,
            parent_id: None,
            lineage_id: None,
            environment_id: environment_id.into(),
            protocol_id: protocol_id.into(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn founder(id: EventId, time: u64, step: u64, env: &str, protocol: &str, label: &str) -> Self {
        let mut event = Self::base(id, time, step, 0, EventType::FounderCreated, env, protocol);
        event.organism_id = label.parse().ok();
        event
    }

    pub fn birth(
        id: EventId,
        time: u64,
        step: u64,
        organism_id: OrganismId,
        parent_id: Option<OrganismId>,
        lineage_id: LineageId,
        env: &str,
        protocol: &str,
    ) -> Self {
        let mut event = Self::base(id, time, step, 0, EventType::Birth, env, protocol);
        event.organism_id = Some(organism_id);
        event.parent_id = parent_id;
        event.lineage_id = Some(lineage_id);
        event
    }

    pub fn death(
        id: EventId,
        time: u64,
        step: u64,
        organism_id: OrganismId,
        replicate: u32,
        env: &str,
        protocol: &str,
    ) -> Self {
        let mut event = Self::base(id, time, step, replicate, EventType::Death, env, protocol);
        event.organism_id = Some(organism_id);
        event
    }

    pub fn experiment_end(id: EventId, time: u64, step: u64, env: &str, protocol: &str) -> Self {
        Self::base(id, time, step, 0, EventType::ExperimentEnd, env, protocol)
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum EventLedgerError {
    #[error("event time moved backwards")]
    TimeMovedBackwards,
    #[error("event step moved backwards")]
    StepMovedBackwards,
    #[error("event id {0} is duplicated")]
    DuplicateEvent(EventId),
    #[error("birth of organism {child} references missing parent {parent}")]
    MissingParent { child: OrganismId, parent: OrganismId },
    #[error("organism {0} was born more than once")]
    DoubleBirth(OrganismId),
    #[error("organism {0} died more than once")]
    DoubleDeath(OrganismId),
    #[error("organism {child} was born before parent {parent}")]
    ChildBeforeParent { child: OrganismId, parent: OrganismId },
    #[error("event ledger serialization failed: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EventLedger {
    pub events: Vec<EventV1>,
    pub next_event_id: EventId,
}

impl EventLedger {
    pub fn append(&mut self, mut event: EventV1) -> Result<EventId, EventLedgerError> {
        if let Some(last) = self.events.last() {
            if event.accepted_simulated_time < last.accepted_simulated_time {
                return Err(EventLedgerError::TimeMovedBackwards);
            }
            if event.accepted_step < last.accepted_step {
                return Err(EventLedgerError::StepMovedBackwards);
            }
        }
        if event.event_id == 0 {
            self.next_event_id += 1;
            event.event_id = self.next_event_id;
        } else if self.events.iter().any(|e| e.event_id == event.event_id) {
            return Err(EventLedgerError::DuplicateEvent(event.event_id));
        } else {
            self.next_event_id = self.next_event_id.max(event.event_id);
        }
        let id = event.event_id;
        self.events.push(event);
        Ok(id)
    }

    pub fn validate(&self) -> Result<(), EventLedgerError> {
        let mut ids = BTreeSet::new();
        let mut born = BTreeSet::new();
        let mut died = BTreeSet::new();
        let mut birth_time = BTreeMap::new();
        for pair in self.events.windows(2) {
            if pair[1].accepted_simulated_time < pair[0].accepted_simulated_time {
                return Err(EventLedgerError::TimeMovedBackwards);
            }
            if pair[1].accepted_step < pair[0].accepted_step {
                return Err(EventLedgerError::StepMovedBackwards);
            }
        }
        for event in &self.events {
            if !ids.insert(event.event_id) {
                return Err(EventLedgerError::DuplicateEvent(event.event_id));
            }
            match event.event_type {
                EventType::FounderCreated | EventType::Birth => {
                    if let Some(organism_id) = event.organism_id {
                        if !born.insert(organism_id) {
                            return Err(EventLedgerError::DoubleBirth(organism_id));
                        }
                        if let Some(parent_id) = event.parent_id {
                            if !born.contains(&parent_id) {
                                return Err(EventLedgerError::MissingParent {
                                    child: organism_id,
                                    parent: parent_id,
                                });
                            }
                            birth_time.insert(organism_id, event.accepted_simulated_time);
                            if let Some(parent_time) = birth_time.get(&parent_id) {
                                if event.accepted_simulated_time < *parent_time {
                                    return Err(EventLedgerError::ChildBeforeParent {
                                        child: organism_id,
                                        parent: parent_id,
                                    });
                                }
                            }
                        } else {
                            birth_time.insert(organism_id, event.accepted_simulated_time);
                        }
                    }
                }
                EventType::Death => {
                    if let Some(organism_id) = event.organism_id {
                        if !died.insert(organism_id) {
                            return Err(EventLedgerError::DoubleDeath(organism_id));
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn hash(&self) -> Result<String, ProtocolError> {
        stable_hash(self)
    }
}
