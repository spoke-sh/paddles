use crate::domain::model::{TaskTraceId, TraceRecord, TraceReplay};
use anyhow::Result;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceRecorderCapability {
    Available,
    Unsupported { reason: String },
}

pub trait TraceRecorder: Send + Sync {
    fn capability(&self) -> TraceRecorderCapability;

    fn record(&self, record: TraceRecord) -> Result<()>;

    fn replay(&self, task_id: &TaskTraceId) -> Result<TraceReplay>;
}

#[derive(Default)]
pub struct NoopTraceRecorder;

impl TraceRecorder for NoopTraceRecorder {
    fn capability(&self) -> TraceRecorderCapability {
        TraceRecorderCapability::Available
    }

    fn record(&self, _record: TraceRecord) -> Result<()> {
        Ok(())
    }

    fn replay(&self, task_id: &TaskTraceId) -> Result<TraceReplay> {
        Ok(TraceReplay {
            task_id: task_id.clone(),
            records: Vec::new(),
        })
    }
}
