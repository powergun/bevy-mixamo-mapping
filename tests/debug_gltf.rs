//! Debug test to inspect GLTF structure

use std::path::Path;

#[test]
fn debug_gltf_structure() {
    let path = Path::new("testdata/Sprint.fbx.glb");
    let (document, _buffers, _images) = gltf::import(path).expect("Failed to load GLB");

    println!("\n=== GLTF Document Structure ===\n");

    // List all nodes
    println!("NODES ({}):", document.nodes().count());
    for node in document.nodes() {
        let parent_info = if let Some(skin) = document.skins().find(|s| {
            s.joints().any(|j| j.index() == node.index())
        }) {
            format!(" [in skin '{}']", skin.name().unwrap_or("unnamed"))
        } else {
            String::new()
        };

        println!(
            "  [{}] {:?} - children: {:?}{}",
            node.index(),
            node.name(),
            node.children().map(|c| c.index()).collect::<Vec<_>>(),
            parent_info
        );
    }

    // List skins
    println!("\nSKINS ({}):", document.skins().count());
    for skin in document.skins() {
        println!(
            "  [{}] {:?} - skeleton: {:?}, joints: {:?}",
            skin.index(),
            skin.name(),
            skin.skeleton().map(|s| s.index()),
            skin.joints().map(|j| j.index()).collect::<Vec<_>>()
        );
    }

    // List animations
    println!("\nANIMATIONS ({}):", document.animations().count());
    for anim in document.animations() {
        println!("  [{}] {:?} - {} channels", anim.index(), anim.name(), anim.channels().count());
        for channel in anim.channels() {
            let target = channel.target();
            println!(
                "    -> node {} ({:?}), property: {:?}",
                target.node().index(),
                target.node().name(),
                target.property()
            );
        }
    }

    // List scenes
    println!("\nSCENES ({}):", document.scenes().count());
    for scene in document.scenes() {
        println!(
            "  [{}] {:?} - root nodes: {:?}",
            scene.index(),
            scene.name(),
            scene.nodes().map(|n| n.index()).collect::<Vec<_>>()
        );
    }
}
