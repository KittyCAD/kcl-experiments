use std::cell::OnceCell;
use std::collections::HashMap;

use crate::domain::*;
use crate::engine_api;

/// Lazy builder for PartDefinition for testing purposes
pub struct KclPart<LazyTopologyBlock: Fn() -> Body> {
    block: LazyTopologyBlock,
    part: OnceCell<PartDefinition>,
}

impl<Constructor: Fn() -> Body> KclPart<Constructor> {
    pub fn new(block: Constructor) -> Self {
        Self {
            block,
            part: OnceCell::new(),
        }
    }

    pub fn definition(&self) -> &PartDefinition {
        // A given Part should only execute its topology one time for all instances.
        self.part.get_or_init(|| {
            let body = (self.block)();
            engine_api::define_part(body)
        })
    }
}

/// actual app logic
impl PartDefinition {
    pub fn instance(&self) -> PartInstance {
        // The body topology exists: now make an instance of it and yield an ID back to KCL.
        let instance_id = engine_api::instance_part(&self.id);
        PartInstance {
            part_id: self.id,
            instance_id,
            body: self.body.id,
            // start the anchor list for this instance with whatever was defined on the body during modeling.
            anchors: self.body.anchors.clone(),
        }
    }
}

// make describing assembly blocks easier
trait AssemblyBlock: Fn() -> AssemblyDefinition {}
impl<F> AssemblyBlock for F where F: Fn() -> AssemblyDefinition {}

/// Lazy builder for AssemblyDefinition for testing purposes
#[derive(Clone)]
pub struct KclAssembly<'a, Block: AssemblyBlock> {
    assembly_id: AssemblyId,
    name: &'a str,
    block: Block,
    definition: OnceCell<AssemblyDefinition>,
}

impl<'a, Block: AssemblyBlock> KclAssembly<'a, Block> {
    pub fn new(name: &'a str, block: Block) -> KclAssembly<'a, Block> {
        KclAssembly {
            assembly_id: AssemblyId::new(),
            name,
            block,
            definition: OnceCell::new(),
        }
    }

    pub fn definition(&self) -> &AssemblyDefinition {
        self.definition.get_or_init(|| (self.block)())
    }
}

impl AssemblyDefinition {
    pub fn new(
        assembly_id: AssemblyId,
        part_instances: Vec<PartInstanceId>,
        instanced_subassemblies: Vec<AssemblyInstanceId>,
        imported_assembly_definitions: HashMap<AssemblyInstanceId, AssemblyDefinition>,
        anchors: Vec<Anchor>,
        mates: Vec<Mate>,
        poses: Vec<Pose>,
    ) -> Self {
        Self {
            assembly_id,
            part_instances,
            instanced_subassemblies,
            imported_assembly_definitions,
            anchors,
            mates,
            poses,
        }
    }

    fn solve<F>(&self, pose_fn: F) -> AssemblyInstance
    where
        F: Fn(&[Pose]) -> &Pose,
    {
        let pose = pose_fn(&self.poses);
        // we go to the engine and ask for fresh instances of the assembly input data (components, anchors, mates)
        let instance_id = engine_api::instance_assembly(self);
        let pose = engine_api::solve(instance_id);
        let mut subassemblies = self.instanced_subassemblies.clone();
        let mut imported_subassemblies = self
            .imported_assembly_definitions
            .iter()
            .map(|(instance_id, _definition)| *instance_id)
            .collect::<Vec<AssemblyInstanceId>>();
        subassemblies.append(&mut imported_subassemblies);
        AssemblyInstance {
            assembly_id: self.assembly_id,
            instance_id,
            anchors: self.anchors.clone(),
            parts: self.part_instances.clone(),
            subassemblies,
            mates: self.mates.clone(),
            solve_result: pose,
        }
    }
}

impl PartInstanceId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl AssemblyInstanceId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl EntityId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Body {
    pub fn new() -> Self {
        Self {
            id: EntityId::new(),
            topology: (),
            anchors: Vec::new(),
        }
    }
}
impl AssemblyId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
impl PartId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
impl PoseId {
    fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Pose {
    pub fn new() -> Self {
        Self {
            id: PoseId::new(),
            sparse_transforms: HashMap::new(),
            overrides: None,
        }
    }
}

impl std::ops::Add for Pose {
    type Output = Self;
    fn add(self, mut rhs: Self) -> Self::Output {
        // note that new_pose _is_ my_sparse_adjustment but it now knows to be applied onto existing_pose
        // new_pose = existing_pose + my_sparse_adjustment
        rhs.overrides = Some(Box::new(self));
        rhs
    }
}

impl std::ops::AddAssign for Pose {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

impl Mate {
    pub fn new(a: &Anchor, b: &Anchor) -> Self {
        Self {
            a: a.clone(),
            a_target: a.on,
            b: b.clone(),
            b_target: b.on,
        }
    }
}

impl Anchor {
    pub fn new(on: PartInstanceId) -> Self {
        // lookup PartInstanceId in a graph, find the topology available, and build a query for it.
        Self {
            id: EntityId::new(),
            on,
            flip_z: false,
        }
    }
}

impl From<&PartInstance> for Anchor {
    fn from(value: &PartInstance) -> Self {
        Self::new(value.instance_id)
    }
}
