use std::sync::{Arc, Mutex};

use derivative::Derivative;

use crate::{metadata::MetaData, sys::aiNode, *};

#[derive(Default, Derivative)]
#[derivative(Debug)]
pub struct Node {
    pub name: String,
    pub children: Vec<Arc<Mutex<Node>>>,
    pub meshes: Vec<u32>,
    pub metadata: Option<MetaData>,
    pub transformation: Matrix4x4,
    #[derivative(Debug = "ignore")]
    pub parent: Option<Arc<Mutex<Node>>>,
}

impl Node {
    pub(crate) fn new(node: &aiNode) -> Arc<Mutex<Node>> {
        Self::allocate(node, None)
    }

    fn allocate(node: &aiNode, parent: Option<&Arc<Mutex<Node>>>) -> Arc<Mutex<Node>> {
        // current simple node
        let res_node = Arc::new(Mutex::new(Self::create_simple_node(node, parent)));
        let res_node_clone = res_node.clone();
        let mut res_node = res_node.lock().unwrap();

        res_node.children = utils::get_base_type_vec_from_raw(node.mChildren, node.mNumChildren)
            .iter()
            .map(|child| Self::allocate(child, Some(&res_node_clone)))
            .collect::<Vec<_>>();

        res_node_clone
    }

    fn create_simple_node(node: &aiNode, parent: Option<&Arc<Mutex<Node>>>) -> Node {
        Node {
            name: node.mName.into(),
            children: Vec::new(),
            meshes: utils::get_raw_vec(node.mMeshes, node.mNumMeshes),
            metadata: utils::get_raw(node.mMetaData),
            transformation: (&node.mTransformation).into(),
            parent: parent.map(|p| p.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::utils;

    #[test]
    fn checking_nodes() {
        use crate::scene::{PostProcess, Scene};

        let current_directory_buf = utils::get_model("models/BLEND/box.blend");

        let scene = Scene::from_file(
            current_directory_buf.as_str(),
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
            ],
        )
        .unwrap();

        let root = scene.root.as_ref().unwrap();
        let root = root.lock().unwrap();
        let children = &root.children;

        assert_eq!("<BlenderRoot>".to_string(), root.name);
        assert_eq!(3, children.len());

        let first_son = &children[0].lock().unwrap();
        assert_eq!("Cube".to_string(), first_son.name);

        let second_son = &children[1].lock().unwrap();
        assert_eq!("Lamp".to_string(), second_son.name);

        assert_eq!(0, root.meshes.len());

        assert!(root.metadata.is_none());

        assert_eq!(1.0, root.transformation.a1);
        assert_eq!(1.0, root.transformation.b2);
        assert_eq!(1.0, root.transformation.c3);
        assert_eq!(1.0, root.transformation.d4);
    }

    #[test]
    fn childs_parent_name_matches() {
        use crate::scene::{PostProcess, Scene};

        let current_directory_buf = utils::get_model("models/BLEND/box.blend");

        let scene = Scene::from_file(
            current_directory_buf.as_str(),
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
            ],
        )
        .unwrap();

        let root = scene.root.as_ref().unwrap();
        let root = root.lock().unwrap();
        let children = &root.children;

        let first_son = &children[0].lock().unwrap();
        let dad = first_son.parent.as_ref().unwrap().lock().unwrap();

        assert_eq!(root.name, dad.name);
    }

    #[test]
    fn debug_root() {
        use crate::scene::{PostProcess, Scene};

        let current_directory_buf = utils::get_model("models/BLEND/box.blend");

        let scene = Scene::from_file(
            current_directory_buf.as_str(),
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
            ],
        )
        .unwrap();

        dbg!(&scene.root);
    }
}
