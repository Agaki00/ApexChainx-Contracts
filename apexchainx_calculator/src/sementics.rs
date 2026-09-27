//! Legacy protocol fixture samples, compiled for compatibility.
//!
//! These sample calculation/configuration events are not the SLA contract ABI.
//! The live event catalog and versions are defined in `event_schema.rs`.

use alloc::{vec, vec::Vec};
#[path = "fixtures/protocol_versions.rs"]
mod protocol_versions;
use protocol_versions::ProtocolVersion;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldenEventFixture {
    pub protocol: ProtocolVersion,
    pub topic: &'static str,
    pub payload: &'static str,
}

pub const CALCULATION_COMPLETED_V1: GoldenEventFixture = GoldenEventFixture {
    protocol: ProtocolVersion::V1,
    topic: "calculation_completed",
    payload: r#"{
  "operation":"add",
  "lhs":4,
  "rhs":6,
  "result":10
}"#,
};

pub const CONFIG_UPDATED_V1: GoldenEventFixture = GoldenEventFixture {
    protocol: ProtocolVersion::V1,
    topic: "config_updated",
    payload: r#"{
  "version":"1"
}"#,
};

pub fn all_v1_fixtures() -> Vec<GoldenEventFixture> {
    vec![CALCULATION_COMPLETED_V1, CONFIG_UPDATED_V1]
}
