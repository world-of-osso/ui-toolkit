//! Atlas regions from the `UiTextureAtlas`, `UiTextureAtlasElement` and
//! `UiTextureAtlasMember` DB2 CSV exports, keyed by element name and
//! `UiTextureAtlasSetID`.
//!
//! Retail tables have no set column: every Retail member is set 0. Forever (WoW Classic
//! "Camelot") adds set-1 members (`*c60` textures) to the names Retail already has; its
//! client resolves set 1 first and set 0 for names it does not re-skin.

use std::collections::HashMap;
use std::path::Path;

use super::{AtlasRegion, AtlasSource};

/// Which atlas set the client draws: Retail art, or Forever's set-1 re-skin over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveSkin {
    #[default]
    Modern,
    Forever,
}

impl ActiveSkin {
    /// `UiTextureAtlasSetID`s in resolution order.
    fn sets(self) -> &'static [u32] {
        match self {
            Self::Modern => &[0],
            Self::Forever => &[1, 0],
        }
    }
}

const FOREVER_SET: u32 = 1;

/// One member that can draw an element: its set, canvas and region.
#[derive(Debug, Clone, Copy)]
struct Member {
    id: u32,
    set: u32,
    canvas: u32,
    region: AtlasRegion,
}

impl Member {
    /// Order members of one set: the 1x canvas (`UiCanvasID` 1) the client's Retail
    /// art uses, then the lowest canvas; among equals the newest (highest) member id.
    fn rank(&self) -> (bool, u32, std::cmp::Reverse<u32>) {
        (self.canvas != 1, self.canvas, std::cmp::Reverse(self.id))
    }
}

#[derive(Debug, Default)]
pub(super) struct AtlasTable {
    /// Lower-cased element name -> its members over all loaded sets.
    members: HashMap<String, Vec<Member>>,
    /// Retail element id -> lower-cased element name.
    names: HashMap<u32, String>,
}

impl AtlasTable {
    /// Retail tables in `retail_dir` (set 0) plus the set-1 members of the Forever
    /// tables in `forever_dir`.
    pub(super) fn load(retail_dir: &Path, forever_dir: &Path) -> Result<Self, String> {
        let mut table = Self::default();
        table.add(&DirTables::read(retail_dir)?, |_| true)?;
        table.add(&DirTables::read(forever_dir)?, |set| set == FOREVER_SET)?;
        Ok(table)
    }

    fn add(&mut self, tables: &DirTables, wanted: impl Fn(u32) -> bool) -> Result<(), String> {
        let elements = parse_elements(&tables.elements)?;
        let atlases = parse_atlases(&tables.atlases)?;
        for row in rows(&tables.members, MEMBER_COLUMNS)? {
            let [
                element,
                atlas,
                id,
                width,
                height,
                left,
                right,
                top,
                bottom,
                ow,
                oh,
            ] = parse_u32s::<11>(&row.fields, &row.line)?;
            let atlas = atlases
                .get(&atlas)
                .ok_or_else(|| format!("UiTextureAtlasMember {id}: unknown atlas {atlas}"))?;
            if !wanted(atlas.set) {
                continue;
            }
            let name = elements
                .get(&element)
                .ok_or_else(|| format!("UiTextureAtlasMember {id}: unknown element {element}"))?;
            let size = [
                if ow == 0 { width } else { ow },
                if oh == 0 { height } else { oh },
            ];
            let region = pixels(atlas, [left, top, right, bottom], size);
            let member = Member {
                id,
                set: atlas.set,
                canvas: atlas.canvas,
                region,
            };
            self.members.entry(name.clone()).or_default().push(member);
            if atlas.set == 0 {
                self.names.entry(element).or_insert_with(|| name.clone());
            }
        }
        Ok(())
    }

    pub(super) fn resolve(&self, name: &str, skin: ActiveSkin) -> Option<AtlasRegion> {
        let members = self.members.get(name)?;
        skin.sets().iter().find_map(|&set| {
            members
                .iter()
                .filter(|member| member.set == set)
                .min_by_key(|member| member.rank())
                .map(|member| member.region)
        })
    }

    pub(super) fn name_of(&self, element_id: u32) -> Option<&str> {
        self.names.get(&element_id).map(String::as_str)
    }
}

/// The three CSV texts of one DB2 export directory.
struct DirTables {
    atlases: String,
    elements: String,
    members: String,
}

impl DirTables {
    fn read(dir: &Path) -> Result<Self, String> {
        let read = |file: &str| {
            let path = dir.join(file);
            std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))
        };
        Ok(Self {
            atlases: read("UiTextureAtlas.csv")?,
            elements: read("UiTextureAtlasElement.csv")?,
            members: read("UiTextureAtlasMember.csv")?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct Atlas {
    fdid: u32,
    set: u32,
    size: [u32; 2],
    canvas: u32,
}

const MEMBER_COLUMNS: &[&str] = &[
    "UiTextureAtlasElementID",
    "UiTextureAtlasID",
    "ID",
    "Width",
    "Height",
    "CommittedLeft",
    "CommittedRight",
    "CommittedTop",
    "CommittedBottom",
    "OverrideWidth",
    "OverrideHeight",
];

fn pixels(atlas: &Atlas, rect: [u32; 4], size: [u32; 2]) -> AtlasRegion {
    let [left, top, right, bottom] = rect;
    let [sheet_width, sheet_height] = atlas.size;
    AtlasRegion {
        source: AtlasSource::FileDataId(atlas.fdid),
        left: left as f32 / sheet_width as f32,
        right: right as f32 / sheet_width as f32,
        top: top as f32 / sheet_height as f32,
        bottom: bottom as f32 / sheet_height as f32,
        width: size[0] as f32,
        height: size[1] as f32,
        tiles_horizontally: false,
        tiles_vertically: false,
        nine_slice_edge: None,
    }
}

fn parse_elements(text: &str) -> Result<HashMap<u32, String>, String> {
    rows(text, &["Name", "ID"])?
        .into_iter()
        .map(|row| {
            let [id] = parse_u32s::<1>(&row.fields[1..], &row.line)?;
            Ok((id, row.fields[0].to_ascii_lowercase()))
        })
        .collect()
}

/// Atlases by id; a table without `UiTextureAtlasSetID` (Retail) is all set 0.
fn parse_atlases(text: &str) -> Result<HashMap<u32, Atlas>, String> {
    let has_set = header(text).contains(&"UiTextureAtlasSetID");
    let columns: &[&str] = if has_set {
        &[
            "ID",
            "FileDataID",
            "AtlasWidth",
            "AtlasHeight",
            "UiCanvasID",
            "UiTextureAtlasSetID",
        ]
    } else {
        &[
            "ID",
            "FileDataID",
            "AtlasWidth",
            "AtlasHeight",
            "UiCanvasID",
        ]
    };
    rows(text, columns)?
        .into_iter()
        .map(|row| {
            let [id, fdid, width, height, canvas] = parse_u32s::<5>(&row.fields[..5], &row.line)?;
            let set = match row.fields.get(5) {
                Some(set) => parse_u32s::<1>(std::slice::from_ref(set), &row.line)?[0],
                None => 0,
            };
            let atlas = Atlas {
                fdid,
                set,
                size: [width, height],
                canvas,
            };
            Ok((id, atlas))
        })
        .collect()
}

struct Row {
    line: String,
    fields: Vec<String>,
}

fn header(text: &str) -> Vec<&str> {
    text.lines().next().unwrap_or_default().split(',').collect()
}

/// `columns` of every data row, in that order. Fields may be double-quoted (names with
/// spaces); no exported atlas field contains a comma.
fn rows(text: &str, columns: &[&str]) -> Result<Vec<Row>, String> {
    let header = header(text);
    let indices = columns
        .iter()
        .map(|column| {
            header
                .iter()
                .position(|name| name == column)
                .ok_or_else(|| format!("CSV has no {column} column"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    text.lines()
        .skip(1)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let cells: Vec<&str> = line.split(',').collect();
            if cells.len() != header.len() {
                return Err(format!(
                    "CSV row has {} of {} fields: {line}",
                    cells.len(),
                    header.len()
                ));
            }
            let fields = indices
                .iter()
                .map(|&index| cells[index].trim_matches('"').to_string())
                .collect();
            Ok(Row {
                line: line.to_string(),
                fields,
            })
        })
        .collect()
}

fn parse_u32s<const N: usize>(fields: &[String], line: &str) -> Result<[u32; N], String> {
    let values = fields
        .iter()
        .take(N)
        .map(|field| field.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("CSV row {line}: {error}"))?;
    values
        .try_into()
        .map_err(|_| format!("CSV row {line}: expected {N} numbers"))
}
