#[allow(unused)]
#[cfg(feature = "debug_gltf")]
pub fn pretty_gltf_tree<P: AsRef<std::path::Path>>(path: P) {
    use std::collections::HashMap;
    use std::sync::atomic::Ordering::Relaxed;

    let mut indent = 0;

    let gltf = gltf::Gltf::open(&path).unwrap();
    let gltf = gltf.document;

    let path = path.as_ref().display();

    println!();
    println!("Parsing file {path}");

    let mut node_parent = HashMap::new();

    for node in gltf.nodes() {
        for child in node.children() {
            node_parent.insert(child.index(), node.index());
        }
    }

    static mut INDENT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    macro_rules! indent {
        () => {
            "|    ".repeat(unsafe { INDENT.load(Relaxed) })
        };
    }

    fn handle_mesh(mesh: Option<gltf::Mesh>) {
        unsafe { INDENT.fetch_add(1, Relaxed) };
        if let Some(mesh) = mesh {
            print_mesh(&mesh);
            for primitive in mesh.primitives() {
                let material = primitive.material();
            }
        } else {
            println!("{}Mesh: None", indent!());
        }
        unsafe { INDENT.fetch_sub(1, Relaxed) };
    }

    fn handle_node(node: &gltf::Node, node_parent: &HashMap<usize, usize>) {
        unsafe { INDENT.fetch_add(1, Relaxed) };
        print_node(&node, &node_parent);

        for child in node.children() {
            handle_node(&child, &node_parent);
        }
        handle_mesh(node.mesh());

        unsafe { INDENT.fetch_sub(1, Relaxed) };
    }

    fn print_node(node: &gltf::Node, node_parent: &HashMap<usize, usize>) {
        let index = node.index();
        let name = node.name().unwrap_or("None");
        let children_count = node.children().count();
        let children_ids = node.children().map(|n| n.index()).collect::<Vec<_>>();

        let parent_index = node_parent.get(&node.index()).copied();
        let parent_str = if let Some(parent_index) = parent_index {
            format!("{}", parent_index)
        } else {
            String::from("None")
        };

        println!(r#"{}Node#{index}: "{name}""#, indent!());
        println!(r#"{}.parent: {parent_str}"#, indent!());
        println!(
            r#"{}.children({children_count}): {children_ids:?}"#,
            indent!()
        );
    }

    fn print_mesh(mesh: &gltf::Mesh) {
        let name = mesh.name().unwrap_or("None");
        let primitives_count = mesh.primitives().count();

        println!(
            r#"{}Mesh: "{name}" with {primitives_count} primitives"#,
            indent!()
        );

        for primitive in mesh.primitives() {
            let material = primitive.material();
        }
    }

    for scene in gltf.scenes() {
        unsafe { INDENT.store(0, Relaxed) };

        let name = scene.name().unwrap_or("None");
        let children_count = scene.nodes().count();
        println!(r#"Scene: "{name}" with {children_count} children"#);

        for node in scene.nodes() {
            handle_node(&node, &node_parent);
        }
    }
}
