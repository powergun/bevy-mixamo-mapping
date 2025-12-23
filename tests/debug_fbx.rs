//! Debug test to inspect FBX structure

#[test]
fn debug_fbx_structure() {
    let opts = ufbx::LoadOpts::default();
    let scene = ufbx::load_file("testdata/Sprint.fbx", opts).expect("Failed to load FBX");

    println!("\n=== FBX Scene Structure ===\n");

    // List all nodes
    println!("NODES ({}):", scene.nodes.len());
    for node in &scene.nodes {
        if node.is_root {
            println!("  [ROOT] {:?}", node.element.name);
            continue;
        }

        let parent_name = node.parent.as_ref()
            .map(|p| p.element.name.as_ref())
            .unwrap_or("(none)");

        // Check if node has a bone attribute
        let bone_marker = if node.bone.is_some() { " [BONE]" } else { "" };

        println!(
            "  [{}] {:?} (parent: {:?}){}",
            node.element.typed_id,
            node.element.name,
            parent_name,
            bone_marker,
        );
    }

    // List bones specifically
    println!("\nBONES ({}):", scene.bones.len());
    for bone in &scene.bones {
        println!("  {:?} - radius: {:.2}", bone.element.name, bone.radius);
    }

    // List skins
    println!("\nSKIN DEFORMERS ({}):", scene.skin_deformers.len());
    for skin in &scene.skin_deformers {
        println!(
            "  {:?} - {} clusters",
            skin.element.name,
            skin.clusters.len()
        );
        for cluster in &skin.clusters {
            println!("    -> {:?}", cluster.bone_node.as_ref().map(|n| n.element.name.as_ref()));
        }
    }

    // List animation stacks
    println!("\nANIMATION STACKS ({}):", scene.anim_stacks.len());
    for stack in &scene.anim_stacks {
        println!(
            "  {:?} - duration: {:.2}s",
            stack.element.name,
            stack.time_end - stack.time_begin
        );

        for layer in &stack.layers {
            println!("    Layer: {:?} - {} values", layer.element.name, layer.anim_values.len());
        }
    }

    println!("\n=== End FBX Structure ===\n");
}
