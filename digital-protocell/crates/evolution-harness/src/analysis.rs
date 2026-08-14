use crate::{EventLedger, LineageTracker, ReplicateResultV1};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrganismAnalysisRow {
    pub replicate: u32,
    pub organism_id: u64,
    pub parent_id: Option<u64>,
    pub lineage_id: u64,
    pub generation: u32,
    pub birth_time: f64,
    pub death_time: Option<f64>,
    pub final_phenotype: String,
    pub final_hereditary_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineageAnalysisRow {
    pub lineage_id: u64,
    pub founder: u64,
    pub max_generation: u32,
    pub births: u64,
    pub deaths: u64,
    pub descendant_count: u64,
    pub extinction_time: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisBundleV1 {
    pub schema: String,
    pub organisms: Vec<OrganismAnalysisRow>,
    pub events: Vec<crate::EventV1>,
    pub lineage: Vec<LineageAnalysisRow>,
    pub replicate: ReplicateResultV1,
}

pub struct AnalysisExporter;

impl AnalysisExporter {
    pub fn event_jsonl(ledger: &EventLedger) -> Result<String, serde_json::Error> {
        ledger.events.iter().map(serde_json::to_string).collect::<Result<Vec<_>, _>>().map(|rows| rows.join("\n"))
    }

    pub fn bundle_json(bundle: &AnalysisBundleV1) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(bundle)
    }

    pub fn lineage_rows(tracker: &LineageTracker) -> Vec<LineageAnalysisRow> {
        let mut rows = Vec::new();
        for record in tracker.ancestry.values().filter(|record| record.parent_id.is_none()) {
            let members = tracker.lineage_members(record.lineage_id);
            let max_generation = members.iter().filter_map(|id| tracker.generation(*id)).max().unwrap_or(0);
            let deaths = members.iter().filter_map(|id| tracker.ancestry.get(id)).filter(|record| record.death_time.is_some()).count() as u64;
            let extinction_time = members
                .iter()
                .filter_map(|id| tracker.ancestry.get(id).and_then(|record| record.death_time))
                .reduce(f64::max);
            rows.push(LineageAnalysisRow {
                lineage_id: record.lineage_id,
                founder: record.organism_id,
                max_generation,
                births: members.len() as u64,
                deaths,
                descendant_count: tracker.descendant_count(record.organism_id) as u64,
                extinction_time,
            });
        }
        rows
    }
}
