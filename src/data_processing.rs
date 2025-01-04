use crate::args::Args;
use crate::block_definitions::{DIRT, GRASS_BLOCK, SNOW_BLOCK};
use crate::element_processing::*;
use crate::osm_parser::ProcessedElement;
use crate::progress::emit_gui_progress_update;
use crate::world_editor::WorldEditor;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

pub fn generate_world(
    elements: Vec<ProcessedElement>,
    args: &Args,
    scale_factor_x: f64,
    scale_factor_z: f64,
) -> Result<(), String> {
    println!("{} Processing data...", "[3/5]".bold());
    emit_gui_progress_update(10.0, "Processing data...");

    let ground_level: i32 = args.ground_level;
    let region_dir: String = format!("{}/region", args.path);
    let mut editor: WorldEditor =
        WorldEditor::new(&region_dir, scale_factor_x, scale_factor_z, args);

    editor.set_sign(
        "↑".to_string(),
        "Generated World".to_string(),
        "This direction".to_string(),
        "".to_string(),
        9,
        -61,
        9,
        6,
    );

    // Process data
    let elements_count: usize = elements.len();
    let process_pb: ProgressBar = ProgressBar::new(elements_count as u64);
    process_pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:45.white/black}] {pos}/{len} elements ({eta}) {msg}")
        .unwrap()
        .progress_chars("█▓░"));

    let progress_increment_prcs: f64 = 50.0 / elements_count as f64;
    let mut current_progress_prcs: f64 = 10.0;
    let mut last_emitted_progress: f64 = current_progress_prcs;
    for element in &elements {
        process_pb.inc(1);
        current_progress_prcs += progress_increment_prcs;
        if (current_progress_prcs - last_emitted_progress).abs() > 0.25 {
            emit_gui_progress_update(current_progress_prcs, "");
            last_emitted_progress = current_progress_prcs;
        }

        if args.debug {
            process_pb.set_message(format!(
                "(Element ID: {} / Type: {})",
                element.id(),
                element.kind()
            ));
        } else {
            process_pb.set_message("");
        }

        match element {
            ProcessedElement::Way(way) => {
                if way.tags.contains_key("building") || way.tags.contains_key("building:part") {
                    buildings::generate_buildings(&mut editor, way, ground_level, args);
                } else if way.tags.contains_key("highway") {
                    highways::generate_highways(&mut editor, element, ground_level, args);
                } else if way.tags.contains_key("landuse") {
                    landuse::generate_landuse(&mut editor, way, ground_level, args);
                } else if way.tags.contains_key("natural") {
                    natural::generate_natural(&mut editor, element, ground_level, args);
                } else if way.tags.contains_key("amenity") {
                    amenities::generate_amenities(&mut editor, element, ground_level, args);
                } else if way.tags.contains_key("leisure") {
                    leisure::generate_leisure(&mut editor, way, ground_level, args);
                } else if way.tags.contains_key("barrier") {
                    barriers::generate_barriers(&mut editor, element, ground_level);
                } else if way.tags.contains_key("waterway") {
                    waterways::generate_waterways(&mut editor, way, ground_level);
                } else if way.tags.contains_key("bridge") {
                    bridges::generate_bridges(&mut editor, way, ground_level);
                } else if way.tags.contains_key("railway") {
                    railways::generate_railways(&mut editor, way, ground_level);
                } else if way.tags.get("service") == Some(&"siding".to_string()) {
                    highways::generate_siding(&mut editor, way, ground_level);
                }
            }
            ProcessedElement::Node(node) => {
                if node.tags.contains_key("door") || node.tags.contains_key("entrance") {
                    doors::generate_doors(&mut editor, node, ground_level);
                } else if node.tags.contains_key("natural")
                    && node.tags.get("natural") == Some(&"tree".to_string())
                {
                    natural::generate_natural(&mut editor, element, ground_level, args);
                } else if node.tags.contains_key("amenity") {
                    amenities::generate_amenities(&mut editor, element, ground_level, args);
                } else if node.tags.contains_key("barrier") {
                    barriers::generate_barriers(&mut editor, element, ground_level);
                } else if node.tags.contains_key("highway") {
                    highways::generate_highways(&mut editor, element, ground_level, args);
                } else if node.tags.contains_key("tourism") {
                    tourisms::generate_tourisms(&mut editor, node, ground_level);
                }
            }
            ProcessedElement::Relation(rel) => {
                if rel.tags.contains_key("building") || rel.tags.contains_key("building:part") {
                    buildings::generate_building_from_relation(
                        &mut editor,
                        rel,
                        ground_level,
                        args,
                    );
                } else if rel.tags.contains_key("water") {
                    water_areas::generate_water_areas(&mut editor, rel, ground_level);
                }
            }
        }
    }

    process_pb.finish();

    // Generate ground layer
    let total_blocks: u64 = (scale_factor_x as i32 + 1) as u64 * (scale_factor_z as i32 + 1) as u64;
    let desired_updates: u64 = 1500;
    let batch_size: u64 = (total_blocks / desired_updates).max(1);

    let mut block_counter: u64 = 0;

    println!("{} Generating ground layer...", "[4/5]".bold());
    emit_gui_progress_update(60.0, "Generating ground layer...");

    let ground_pb: ProgressBar = ProgressBar::new(total_blocks);
    ground_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:45}] {pos}/{len} blocks ({eta})")
            .unwrap()
            .progress_chars("█▓░"),
    );

    let mut gui_progress_grnd: f64 = 60.0;
    let mut last_emitted_progress: f64 = gui_progress_grnd;
    let total_iterations_grnd: f64 = (scale_factor_x + 1.0) * (scale_factor_z + 1.0);
    let progress_increment_grnd: f64 = 30.0 / total_iterations_grnd;

    let groundlayer_block = if args.winter { SNOW_BLOCK } else { GRASS_BLOCK };

    for x in 0..=(scale_factor_x as i32) {
        for z in 0..=(scale_factor_z as i32) {
            editor.set_block(groundlayer_block, x, ground_level, z, None, None);
            editor.set_block(DIRT, x, ground_level - 1, z, None, None);

            block_counter += 1;
            if block_counter % batch_size == 0 {
                ground_pb.inc(batch_size);
            }

            gui_progress_grnd += progress_increment_grnd;
            if (gui_progress_grnd - last_emitted_progress).abs() > 0.25 {
                emit_gui_progress_update(gui_progress_grnd, "");
                last_emitted_progress = gui_progress_grnd;
            }
        }
    }

    ground_pb.inc(block_counter % batch_size);
    ground_pb.finish();

    // Save world
    editor.save();

    emit_gui_progress_update(100.0, "Done! World generation completed.");
    println!("{}", "Done! World generation completed.".green().bold());
    Ok(())
}
