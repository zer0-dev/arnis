use crate::block_definitions::*;
use crate::cartesian::XZPoint;
use crate::osm_parser::ProcessedNode;
use crate::ground::Ground;
use crate::world_editor::WorldEditor;

pub fn generate_tourisms(editor: &mut WorldEditor, element: &ProcessedNode, ground: &Ground) {
    // Skip if 'layer' or 'level' is negative in the tags
    if let Some(layer) = element.tags.get("layer") {
        if layer.parse::<i32>().unwrap_or(0) < 0 {
            return;
        }
    }

    if let Some(level) = element.tags.get("level") {
        if level.parse::<i32>().unwrap_or(0) < 0 {
            return;
        }
    }

    if let Some(tourism_type) = element.tags.get("tourism") {
        let x: i32 = element.x;
        let z: i32 = element.z;

        // Calculate the dynamic ground level
        let ground_level = ground.level(XZPoint::new(x, z));

        if tourism_type == "information" {
            if let Some("board") = element.tags.get("information").map(|x: &String| x.as_str()) {
                // Draw an information board
                editor.set_block(OAK_PLANKS, x, ground_level + 1, z, None, None);
            }
        }
    }
}
