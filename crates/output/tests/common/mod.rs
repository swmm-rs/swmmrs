#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use swmm_output::{
    LinkResultAttribute, NodeResultAttribute, OutputReader, OutputSeriesSelection,
    SubcatchmentResultAttribute, SystemResultAttribute,
};

pub const LEGACY_SIZE: usize = 225_442;
pub const LEGACY_SHA256: &str = "64a9d61b169dcf6587d954a4d3898483b0c0fdd1d89d328dcf33b6b0b4824cfb";
pub const EXPANDED_SIZE: usize = 1_463;
pub const EXPANDED_SHA256: &str =
    "5d53760cec7e7df32816e710b16a2fb614d8ac8586c4e523313ea3e24876e201";

pub const MAGIC: i32 = 516_114_522;
const HEADER_BYTES: usize = 28;
const TRAILER_BYTES: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixtureKind {
    Legacy,
    Expanded,
}

impl FixtureKind {
    pub const fn relative_path(self) -> &'static str {
        match self {
            Self::Legacy => "legacy/Model.out",
            Self::Expanded => "expanded/minimal.out",
        }
    }

    pub const fn expected_size(self) -> usize {
        match self {
            Self::Legacy => LEGACY_SIZE,
            Self::Expanded => EXPANDED_SIZE,
        }
    }

    pub const fn expected_sha256(self) -> &'static str {
        match self {
            Self::Legacy => LEGACY_SHA256,
            Self::Expanded => EXPANDED_SHA256,
        }
    }

    pub const fn expected_release(self) -> i32 {
        match self {
            Self::Legacy => 51_014,
            Self::Expanded => 53_000,
        }
    }

    pub const fn expected_counts(self) -> Counts {
        match self {
            Self::Legacy => Counts {
                subcatchments: 3,
                nodes: 9,
                links: 8,
                pollutants: 3,
            },
            Self::Expanded => Counts {
                subcatchments: 1,
                nodes: 2,
                links: 1,
                pollutants: 1,
            },
        }
    }

    pub const fn expected_sections(self) -> Sections {
        match self {
            Self::Legacy => Sections {
                identifiers: 28,
                properties: 250,
                results: 778,
                periods: 288,
            },
            Self::Expanded => Sections {
                identifiers: 28,
                properties: 63,
                results: 335,
                periods: 6,
            },
        }
    }
}

pub fn fixture_path(kind: FixtureKind) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(kind.relative_path())
}

pub fn fixture_input_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("expanded/minimal.inp")
}

pub fn provenance_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures/PROVENANCE.toml")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counts {
    pub subcatchments: usize,
    pub nodes: usize,
    pub links: usize,
    pub pollutants: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sections {
    pub identifiers: usize,
    pub properties: usize,
    pub results: usize,
    pub periods: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyBlockLayout {
    pub count_offset: usize,
    pub code_offsets: Vec<usize>,
    pub value_offsets: Vec<usize>,
    pub element_count: usize,
    pub property_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultSchemaLayout {
    pub count_offset: usize,
    pub code_offsets: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureLayout {
    pub sections: Sections,
    pub trailer_offset: usize,
    pub name_length_offsets: Vec<usize>,
    pub name_lengths: Vec<usize>,
    pub name_data_offsets: Vec<usize>,
    pub pollutant_unit_offsets: Vec<usize>,
    pub property_blocks: Vec<PropertyBlockLayout>,
    pub result_schemas: Vec<ResultSchemaLayout>,
    pub schedule_origin_offset: usize,
    pub report_step_offset: usize,
    pub simulation_code_offset: usize,
}

pub fn copy_fixture(kind: FixtureKind) -> (tempfile::TempDir, PathBuf, Vec<u8>) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("patched.out");
    fs::copy(fixture_path(kind), &path).unwrap();
    let bytes = fs::read(&path).unwrap();
    (directory, path, bytes)
}

pub fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

pub fn fixture_layout(bytes: &[u8]) -> FixtureLayout {
    let counts = Counts {
        subcatchments: read_count(bytes, 12),
        nodes: read_count(bytes, 16),
        links: read_count(bytes, 20),
        pollutants: read_count(bytes, 24),
    };
    let trailer_offset = bytes.len() - TRAILER_BYTES;
    let sections = Sections {
        identifiers: read_offset(bytes, trailer_offset),
        properties: read_offset(bytes, trailer_offset + 4),
        results: read_offset(bytes, trailer_offset + 8),
        periods: read_count(bytes, trailer_offset + 12),
    };
    let name_count = counts.subcatchments + counts.nodes + counts.links + counts.pollutants;
    let mut cursor = sections.identifiers;
    let mut name_length_offsets = Vec::with_capacity(name_count);
    let mut name_lengths = Vec::with_capacity(name_count);
    let mut name_data_offsets = Vec::with_capacity(name_count);
    for _ in 0..name_count {
        let length_offset = cursor;
        let length = read_count(bytes, cursor);
        cursor += 4;
        name_length_offsets.push(length_offset);
        name_lengths.push(length);
        name_data_offsets.push(cursor);
        cursor += length;
    }
    let mut pollutant_unit_offsets = Vec::with_capacity(counts.pollutants);
    for _ in 0..counts.pollutants {
        pollutant_unit_offsets.push(cursor);
        cursor += 4;
    }
    assert_eq!(cursor, sections.properties);

    let mut property_blocks = Vec::with_capacity(3);
    for &element_count in &[counts.subcatchments, counts.nodes, counts.links] {
        let count_offset = cursor;
        let property_count = read_count(bytes, cursor);
        cursor += 4;
        let mut code_offsets = Vec::with_capacity(property_count);
        for _ in 0..property_count {
            code_offsets.push(cursor);
            cursor += 4;
        }
        let mut value_offsets = Vec::with_capacity(element_count * property_count);
        for _ in 0..element_count {
            for _ in 0..property_count {
                value_offsets.push(cursor);
                cursor += 4;
            }
        }
        property_blocks.push(PropertyBlockLayout {
            count_offset,
            code_offsets,
            value_offsets,
            element_count,
            property_count,
        });
    }
    let mut result_schemas = Vec::with_capacity(4);
    for _ in 0..4 {
        let count_offset = cursor;
        let count = read_count(bytes, cursor);
        cursor += 4;
        let mut code_offsets = Vec::with_capacity(count);
        for _ in 0..count {
            code_offsets.push(cursor);
            cursor += 4;
        }
        result_schemas.push(ResultSchemaLayout {
            count_offset,
            code_offsets,
        });
    }
    let schedule_origin_offset = cursor;
    let report_step_offset = cursor + 8;
    assert_eq!(cursor + 12, sections.results);
    FixtureLayout {
        sections,
        trailer_offset,
        name_length_offsets,
        name_lengths,
        name_data_offsets,
        pollutant_unit_offsets,
        property_blocks,
        result_schemas,
        schedule_origin_offset,
        report_step_offset,
        simulation_code_offset: trailer_offset + 16,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawElementName {
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawF32 {
    pub bits: u32,
}

impl RawF32 {
    pub const fn value(self) -> f32 {
        f32::from_bits(self.bits)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawSubcatchment {
    pub name: RawElementName,
    pub area: RawF32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawNode {
    pub name: RawElementName,
    pub kind: i32,
    pub invert_elevation: RawF32,
    pub maximum_depth: RawF32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawLink {
    pub name: RawElementName,
    pub kind: i32,
    pub inlet_offset: RawF32,
    pub outlet_offset: RawF32,
    pub maximum_depth: RawF32,
    pub length: RawF32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawPollutant {
    pub name: RawElementName,
    pub concentration_units: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawSchemas {
    pub subcatchment: Vec<i32>,
    pub node: Vec<i32>,
    pub link: Vec<i32>,
    pub system: Vec<i32>,
}

/// Independent decoder for the fixture record format. It intentionally owns
/// its byte parser and physical-column arithmetic instead of using production
/// layout, selection, or read-plan code.
#[derive(Clone, Debug)]
pub struct RawOracle {
    pub kind: FixtureKind,
    pub bytes: Vec<u8>,
    pub solver_release: i32,
    pub flow_units: i32,
    pub counts: Counts,
    pub sections: Sections,
    pub subcatchments: Vec<RawSubcatchment>,
    pub nodes: Vec<RawNode>,
    pub links: Vec<RawLink>,
    pub pollutants: Vec<RawPollutant>,
    pub schemas: RawSchemas,
    pub schedule_origin_bits: u64,
    pub report_step_seconds: i32,
    pub period_count: usize,
    pub bytes_per_period: usize,
    pub result_start: usize,
}

impl RawOracle {
    pub fn load(kind: FixtureKind) -> Self {
        let path = fixture_path(kind);
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_fixture_identity_bytes(kind, &bytes);
        let expected_counts = kind.expected_counts();
        let expected_sections = kind.expected_sections();
        assert_eq!(read_i32(&bytes, 0), MAGIC);
        let solver_release = read_i32(&bytes, 4);
        let flow_units = read_i32(&bytes, 8);
        let counts = Counts {
            subcatchments: read_count(&bytes, 12),
            nodes: read_count(&bytes, 16),
            links: read_count(&bytes, 20),
            pollutants: read_count(&bytes, 24),
        };
        assert_eq!(solver_release, kind.expected_release());
        assert_eq!(counts, expected_counts);

        let trailer = bytes.len() - TRAILER_BYTES;
        let sections = Sections {
            identifiers: read_offset(&bytes, trailer),
            properties: read_offset(&bytes, trailer + 4),
            results: read_offset(&bytes, trailer + 8),
            periods: read_count(&bytes, trailer + 12),
        };
        assert_eq!(sections, expected_sections);
        assert_eq!(read_i32(&bytes, trailer + 16), 0);
        assert_eq!(read_i32(&bytes, trailer + 20), MAGIC);
        assert_eq!(sections.identifiers, HEADER_BYTES);

        let mut cursor = sections.identifiers;
        let name_count = counts.subcatchments + counts.nodes + counts.links + counts.pollutants;
        let mut names = Vec::with_capacity(name_count);
        for _ in 0..name_count {
            names.push(RawElementName {
                bytes: read_name(&bytes, &mut cursor),
            });
        }
        let mut concentration_units = Vec::with_capacity(counts.pollutants);
        for _ in 0..counts.pollutants {
            concentration_units.push(read_i32(&bytes, take(&mut cursor, 4)));
        }
        assert_eq!(cursor, sections.properties);

        let (sub_property_codes, sub_values) =
            read_property_block(&bytes, &mut cursor, counts.subcatchments);
        let (node_property_codes, node_values) =
            read_property_block(&bytes, &mut cursor, counts.nodes);
        let (link_property_codes, link_values) =
            read_property_block(&bytes, &mut cursor, counts.links);
        assert_eq!(
            (
                sub_property_codes.as_slice(),
                node_property_codes.as_slice(),
                link_property_codes.as_slice()
            ),
            match kind {
                FixtureKind::Legacy => (&[1][..], &[0, 2, 3][..], &[0, 4, 4, 3, 5][..]),
                FixtureKind::Expanded => (&[1][..], &[0, 2, 4][..], &[0, 6, 7, 4, 8][..]),
            },
        );

        let schemas = RawSchemas {
            subcatchment: read_schema(&bytes, &mut cursor),
            node: read_schema(&bytes, &mut cursor),
            link: read_schema(&bytes, &mut cursor),
            system: read_schema(&bytes, &mut cursor),
        };
        let schedule_origin_bits = read_u64(&bytes, take(&mut cursor, 8));
        let report_step_seconds = read_i32(&bytes, take(&mut cursor, 4));
        assert_eq!(cursor, sections.results);
        assert_eq!(report_step_seconds, 300);
        let payload_words = counts.subcatchments * schemas.subcatchment.len()
            + counts.nodes * schemas.node.len()
            + counts.links * schemas.link.len()
            + schemas.system.len();
        let bytes_per_period = 8 + payload_words * 4;
        assert_eq!(
            sections.results + sections.periods * bytes_per_period + TRAILER_BYTES,
            bytes.len()
        );

        let mut name_cursor = 0;
        let mut subcatchments = Vec::with_capacity(counts.subcatchments);
        for values in sub_values {
            let area = RawF32 {
                bits: property_f32(&sub_property_codes, &values, 1).to_bits(),
            };
            subcatchments.push(RawSubcatchment {
                name: names[name_cursor].clone(),
                area,
            });
            name_cursor += 1;
        }
        let mut nodes = Vec::with_capacity(counts.nodes);
        for values in node_values {
            let kind_code = property_i32(&node_property_codes, &values, 0);
            let invert_elevation = RawF32 {
                bits: property_f32(&node_property_codes, &values, 2).to_bits(),
            };
            let maximum_depth = RawF32 {
                bits: property_f32(
                    &node_property_codes,
                    &values,
                    match kind {
                        FixtureKind::Legacy => 3,
                        FixtureKind::Expanded => 4,
                    },
                )
                .to_bits(),
            };
            nodes.push(RawNode {
                name: names[name_cursor].clone(),
                kind: kind_code,
                invert_elevation,
                maximum_depth,
            });
            name_cursor += 1;
        }
        let mut links = Vec::with_capacity(counts.links);
        for values in link_values {
            let kind_code = property_i32(&link_property_codes, &values, 0);
            let (inlet_code, outlet_code, outlet_occurrence, maximum_code, length_code) = match kind
            {
                FixtureKind::Legacy => (4, 4, 1, 3, 5),
                FixtureKind::Expanded => (6, 7, 0, 4, 8),
            };
            let inlet_offset = property_f32_at(&link_property_codes, &values, inlet_code, 0);
            let outlet_offset = property_f32_at(
                &link_property_codes,
                &values,
                outlet_code,
                outlet_occurrence,
            );
            let maximum_depth = RawF32 {
                bits: property_f32(&link_property_codes, &values, maximum_code).to_bits(),
            };
            let length = RawF32 {
                bits: property_f32(&link_property_codes, &values, length_code).to_bits(),
            };
            links.push(RawLink {
                name: names[name_cursor].clone(),
                kind: kind_code,
                inlet_offset,
                outlet_offset,
                maximum_depth,
                length,
            });
            name_cursor += 1;
        }
        let mut pollutants = Vec::with_capacity(counts.pollutants);
        for concentration_units in concentration_units {
            pollutants.push(RawPollutant {
                name: names[name_cursor].clone(),
                concentration_units,
            });
            name_cursor += 1;
        }
        assert_eq!(name_cursor, names.len());

        Self {
            kind,
            bytes,
            solver_release,
            flow_units,
            counts,
            sections,
            subcatchments,
            nodes,
            links,
            pollutants,
            schemas,
            schedule_origin_bits,
            report_step_seconds,
            period_count: sections.periods,
            bytes_per_period,
            result_start: sections.results,
        }
    }

    pub fn date_bits(&self, period: usize) -> u64 {
        assert!(period < self.period_count);
        read_u64(
            &self.bytes,
            self.result_start + period * self.bytes_per_period,
        )
    }

    pub fn cell_bits(&self, selection: &OutputSeriesSelection, period: usize) -> u32 {
        assert!(period < self.period_count);
        let byte_offset = match *selection {
            OutputSeriesSelection::Subcatchment { id, attribute } => {
                let column =
                    unique_column(&self.schemas.subcatchment, subcatchment_code(attribute));
                8 + (id.index() * self.schemas.subcatchment.len() + column) * 4
            }
            OutputSeriesSelection::Node { id, attribute } => {
                let column = unique_column(&self.schemas.node, node_code(attribute));
                8 + (self.counts.subcatchments * self.schemas.subcatchment.len()
                    + id.index() * self.schemas.node.len()
                    + column)
                    * 4
            }
            OutputSeriesSelection::Link { id, attribute } => {
                let column = unique_column(&self.schemas.link, link_code(attribute));
                8 + (self.counts.subcatchments * self.schemas.subcatchment.len()
                    + self.counts.nodes * self.schemas.node.len()
                    + id.index() * self.schemas.link.len()
                    + column)
                    * 4
            }
            OutputSeriesSelection::System { attribute } => {
                let column = unique_column(&self.schemas.system, system_code(attribute));
                8 + (self.counts.subcatchments * self.schemas.subcatchment.len()
                    + self.counts.nodes * self.schemas.node.len()
                    + self.counts.links * self.schemas.link.len()
                    + column)
                    * 4
            }
        };
        read_u32(
            &self.bytes,
            self.result_start + period * self.bytes_per_period + byte_offset,
        )
    }

    pub fn all_selections(&self, reader: &OutputReader) -> Vec<OutputSeriesSelection> {
        let metadata = reader.metadata();
        let mut selections = Vec::new();
        for id in 0..self.counts.subcatchments {
            for &code in &self.schemas.subcatchment {
                selections.push(OutputSeriesSelection::Subcatchment {
                    id: metadata.subcatchments()[id].id(),
                    attribute: subcatchment_attribute(code, metadata.pollutants()),
                });
            }
        }
        for id in 0..self.counts.nodes {
            for &code in &self.schemas.node {
                selections.push(OutputSeriesSelection::Node {
                    id: metadata.nodes()[id].id(),
                    attribute: node_attribute(code, metadata.pollutants()),
                });
            }
        }
        for id in 0..self.counts.links {
            for &code in &self.schemas.link {
                selections.push(OutputSeriesSelection::Link {
                    id: metadata.links()[id].id(),
                    attribute: link_attribute(code, metadata.pollutants()),
                });
            }
        }
        for &code in &self.schemas.system {
            selections.push(OutputSeriesSelection::System {
                attribute: system_attribute(code),
            });
        }
        selections
    }
}

pub fn assert_fixture_identity(kind: FixtureKind) {
    let path = fixture_path(kind);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    assert_fixture_identity_bytes(kind, &bytes);
}

fn assert_fixture_identity_bytes(kind: FixtureKind, bytes: &[u8]) {
    assert_eq!(bytes.len(), kind.expected_size());
    assert_eq!(sha256_hex(bytes), kind.expected_sha256());
}

fn read_name(bytes: &[u8], cursor: &mut usize) -> Vec<u8> {
    let length = read_i32(bytes, take(cursor, 4));
    assert!(length >= 0);
    let length = length as usize;
    let start = *cursor;
    let end = start.checked_add(length).expect("name extent overflow");
    assert!(end <= bytes.len());
    *cursor = end;
    bytes[start..end].to_vec()
}

fn read_property_block(
    bytes: &[u8],
    cursor: &mut usize,
    element_count: usize,
) -> (Vec<i32>, Vec<Vec<u32>>) {
    let property_count = read_count(bytes, take(cursor, 4));
    let mut codes = Vec::with_capacity(property_count);
    for _ in 0..property_count {
        codes.push(read_i32(bytes, take(cursor, 4)));
    }
    let mut values = Vec::with_capacity(element_count);
    for _ in 0..element_count {
        let mut row = Vec::with_capacity(property_count);
        for _ in 0..property_count {
            row.push(read_u32(bytes, take(cursor, 4)));
        }
        values.push(row);
    }
    (codes, values)
}

fn read_schema(bytes: &[u8], cursor: &mut usize) -> Vec<i32> {
    let count = read_count(bytes, take(cursor, 4));
    let mut codes = Vec::with_capacity(count);
    for _ in 0..count {
        codes.push(read_i32(bytes, take(cursor, 4)));
    }
    codes
}

fn property_i32(codes: &[i32], values: &[u32], code: i32) -> i32 {
    i32::from_le_bytes(property_word(codes, values, code, 0).to_le_bytes())
}

fn property_f32(codes: &[i32], values: &[u32], code: i32) -> f32 {
    f32::from_bits(property_word(codes, values, code, 0))
}

fn property_f32_at(codes: &[i32], values: &[u32], code: i32, occurrence: usize) -> RawF32 {
    let index = codes
        .iter()
        .enumerate()
        .filter(|(_, value)| **value == code)
        .nth(occurrence)
        .map(|(index, _)| index)
        .expect("property code missing");
    RawF32 {
        bits: values[index],
    }
}

fn property_word(codes: &[i32], values: &[u32], code: i32, occurrence: usize) -> u32 {
    let index = codes
        .iter()
        .enumerate()
        .filter(|(_, value)| **value == code)
        .nth(occurrence)
        .map(|(index, _)| index)
        .expect("property code missing");
    values[index]
}

fn unique_column(schema: &[i32], code: i32) -> usize {
    let mut found = None;
    let mut occurrences = 0;
    for (index, &value) in schema.iter().enumerate() {
        if value == code {
            found = Some(index);
            occurrences += 1;
        }
    }
    assert_eq!(occurrences, 1, "oracle selection must be unambiguous");
    found.unwrap()
}

pub fn subcatchment_attribute(
    code: i32,
    pollutants: &[swmm_output::PollutantMetadata],
) -> SubcatchmentResultAttribute {
    match code {
        0 => SubcatchmentResultAttribute::Rainfall,
        1 => SubcatchmentResultAttribute::SnowDepth,
        2 => SubcatchmentResultAttribute::EvaporationLoss,
        3 => SubcatchmentResultAttribute::InfiltrationLoss,
        4 => SubcatchmentResultAttribute::RunoffFlow,
        5 => SubcatchmentResultAttribute::GroundwaterFlow,
        6 => SubcatchmentResultAttribute::GroundwaterElevation,
        7 => SubcatchmentResultAttribute::SoilMoisture,
        value if value >= 8 && (value as usize - 8) < pollutants.len() => {
            SubcatchmentResultAttribute::Pollutant(pollutants[value as usize - 8].id())
        }
        value => SubcatchmentResultAttribute::Unknown(value),
    }
}

pub fn node_attribute(
    code: i32,
    pollutants: &[swmm_output::PollutantMetadata],
) -> NodeResultAttribute {
    match code {
        0 => NodeResultAttribute::Depth,
        1 => NodeResultAttribute::HydraulicHead,
        2 => NodeResultAttribute::StoredVolume,
        3 => NodeResultAttribute::LateralInflow,
        4 => NodeResultAttribute::TotalInflow,
        5 => NodeResultAttribute::Overflow,
        value if value >= 6 && (value as usize - 6) < pollutants.len() => {
            NodeResultAttribute::Pollutant(pollutants[value as usize - 6].id())
        }
        value => NodeResultAttribute::Unknown(value),
    }
}

pub fn link_attribute(
    code: i32,
    pollutants: &[swmm_output::PollutantMetadata],
) -> LinkResultAttribute {
    match code {
        0 => LinkResultAttribute::Flow,
        1 => LinkResultAttribute::Depth,
        2 => LinkResultAttribute::Velocity,
        3 => LinkResultAttribute::Volume,
        4 => LinkResultAttribute::Capacity,
        value if value >= 5 && (value as usize - 5) < pollutants.len() => {
            LinkResultAttribute::Pollutant(pollutants[value as usize - 5].id())
        }
        value => LinkResultAttribute::Unknown(value),
    }
}

pub fn system_attribute(code: i32) -> SystemResultAttribute {
    match code {
        0 => SystemResultAttribute::AirTemperature,
        1 => SystemResultAttribute::Rainfall,
        2 => SystemResultAttribute::SnowDepth,
        3 => SystemResultAttribute::InfiltrationLoss,
        4 => SystemResultAttribute::RunoffFlow,
        5 => SystemResultAttribute::DryWeatherInflow,
        6 => SystemResultAttribute::GroundwaterInflow,
        7 => SystemResultAttribute::RdiiInflow,
        8 => SystemResultAttribute::ExternalInflow,
        9 => SystemResultAttribute::TotalLateralInflow,
        10 => SystemResultAttribute::FloodingOutflow,
        11 => SystemResultAttribute::OutfallFlow,
        12 => SystemResultAttribute::StorageVolume,
        13 => SystemResultAttribute::Evaporation,
        14 => SystemResultAttribute::PotentialEvapotranspiration,
        value => SystemResultAttribute::Unknown(value),
    }
}

fn subcatchment_code(attribute: SubcatchmentResultAttribute) -> i32 {
    match attribute {
        SubcatchmentResultAttribute::Rainfall => 0,
        SubcatchmentResultAttribute::SnowDepth => 1,
        SubcatchmentResultAttribute::EvaporationLoss => 2,
        SubcatchmentResultAttribute::InfiltrationLoss => 3,
        SubcatchmentResultAttribute::RunoffFlow => 4,
        SubcatchmentResultAttribute::GroundwaterFlow => 5,
        SubcatchmentResultAttribute::GroundwaterElevation => 6,
        SubcatchmentResultAttribute::SoilMoisture => 7,
        SubcatchmentResultAttribute::Pollutant(id) => 8 + id.index() as i32,
        SubcatchmentResultAttribute::Unknown(value) => value,
    }
}

fn node_code(attribute: NodeResultAttribute) -> i32 {
    match attribute {
        NodeResultAttribute::Depth => 0,
        NodeResultAttribute::HydraulicHead => 1,
        NodeResultAttribute::StoredVolume => 2,
        NodeResultAttribute::LateralInflow => 3,
        NodeResultAttribute::TotalInflow => 4,
        NodeResultAttribute::Overflow => 5,
        NodeResultAttribute::Pollutant(id) => 6 + id.index() as i32,
        NodeResultAttribute::Unknown(value) => value,
    }
}

fn link_code(attribute: LinkResultAttribute) -> i32 {
    match attribute {
        LinkResultAttribute::Flow => 0,
        LinkResultAttribute::Depth => 1,
        LinkResultAttribute::Velocity => 2,
        LinkResultAttribute::Volume => 3,
        LinkResultAttribute::Capacity => 4,
        LinkResultAttribute::Pollutant(id) => 5 + id.index() as i32,
        LinkResultAttribute::Unknown(value) => value,
    }
}

fn system_code(attribute: SystemResultAttribute) -> i32 {
    match attribute {
        SystemResultAttribute::AirTemperature => 0,
        SystemResultAttribute::Rainfall => 1,
        SystemResultAttribute::SnowDepth => 2,
        SystemResultAttribute::InfiltrationLoss => 3,
        SystemResultAttribute::RunoffFlow => 4,
        SystemResultAttribute::DryWeatherInflow => 5,
        SystemResultAttribute::GroundwaterInflow => 6,
        SystemResultAttribute::RdiiInflow => 7,
        SystemResultAttribute::ExternalInflow => 8,
        SystemResultAttribute::TotalLateralInflow => 9,
        SystemResultAttribute::FloodingOutflow => 10,
        SystemResultAttribute::OutfallFlow => 11,
        SystemResultAttribute::StorageVolume => 12,
        SystemResultAttribute::Evaporation => 13,
        SystemResultAttribute::PotentialEvapotranspiration => 14,
        SystemResultAttribute::Unknown(value) => value,
    }
}

fn take(cursor: &mut usize, length: usize) -> usize {
    let start = *cursor;
    *cursor = (*cursor)
        .checked_add(length)
        .expect("fixture cursor overflow");
    start
}

fn read_count(bytes: &[u8], offset: usize) -> usize {
    let value = read_i32(bytes, offset);
    assert!(value >= 0);
    value as usize
}

fn read_offset(bytes: &[u8], offset: usize) -> usize {
    read_count(bytes, offset)
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    let end = offset.checked_add(4).expect("fixture read overflow");
    i32::from_le_bytes(bytes[offset..end].try_into().unwrap())
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    let end = offset.checked_add(4).expect("fixture read overflow");
    u32::from_le_bytes(bytes[offset..end].try_into().unwrap())
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    let end = offset.checked_add(8).expect("fixture read overflow");
    u64::from_le_bytes(bytes[offset..end].try_into().unwrap())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = sha256(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(char::from(b"0123456789abcdef"[(byte >> 4) as usize]));
        output.push(char::from(b"0123456789abcdef"[(byte & 0x0f) as usize]));
    }
    output
}

/// Small test-only SHA-256 implementation using only the Rust standard library.
pub fn sha256(input: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state = [
        0x6a09e667_u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let padded_len = (input.len() + 9).next_multiple_of(64);
    let mut padded = Vec::with_capacity(padded_len);
    padded.extend_from_slice(input);
    padded.push(0x80);
    padded.resize(padded_len - 8, 0);
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut schedule = [0_u32; 64];
        for (index, word) in schedule[..16].iter_mut().enumerate() {
            let start = index * 4;
            *word = u32::from_be_bytes(chunk[start..start + 4].try_into().unwrap());
        }
        for index in 16..64 {
            let s0 = schedule[index - 15].rotate_right(7)
                ^ schedule[index - 15].rotate_right(18)
                ^ (schedule[index - 15] >> 3);
            let s1 = schedule[index - 2].rotate_right(17)
                ^ schedule[index - 2].rotate_right(19)
                ^ (schedule[index - 2] >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(s1);
        }
        let mut working = state;
        for index in 0..64 {
            let s1 = working[4].rotate_right(6)
                ^ working[4].rotate_right(11)
                ^ working[4].rotate_right(25);
            let choose = (working[4] & working[5]) ^ ((!working[4]) & working[6]);
            let temp1 = working[7]
                .wrapping_add(s1)
                .wrapping_add(choose)
                .wrapping_add(K[index])
                .wrapping_add(schedule[index]);
            let s0 = working[0].rotate_right(2)
                ^ working[0].rotate_right(13)
                ^ working[0].rotate_right(22);
            let majority =
                (working[0] & working[1]) ^ (working[0] & working[2]) ^ (working[1] & working[2]);
            let temp2 = s0.wrapping_add(majority);
            working[7] = working[6];
            working[6] = working[5];
            working[5] = working[4];
            working[4] = working[3].wrapping_add(temp1);
            working[3] = working[2];
            working[2] = working[1];
            working[1] = working[0];
            working[0] = temp1.wrapping_add(temp2);
        }
        for index in 0..8 {
            state[index] = state[index].wrapping_add(working[index]);
        }
    }

    let mut digest = [0_u8; 32];
    for (index, word) in state.into_iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::{FixtureKind, assert_fixture_identity, sha256_hex};

    #[test]
    fn sha256_known_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_matches_pinned_fixtures() {
        assert_fixture_identity(FixtureKind::Legacy);
        assert_fixture_identity(FixtureKind::Expanded);
    }
}
