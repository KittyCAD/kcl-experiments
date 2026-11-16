use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub struct PartId(pub uuid::Uuid);

// globally unique.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PartInstanceId(pub uuid::Uuid);

// abstract assembly that can yield many concrete poses
#[derive(Copy, Clone, Debug)]
pub struct AssemblyId(pub uuid::Uuid);

// globally unique.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AssemblyInstanceId(pub uuid::Uuid);

/// This is an engine "entityId"
#[derive(Clone, Copy, Debug)]
pub struct EntityId(pub uuid::Uuid);

#[derive(Clone)]
pub struct Body {
    pub id: EntityId,
    pub topology: (),
    /// not MVP. modeling Anchors.
    pub anchors: Vec<Anchor>,
}

pub struct PartDefinition {
    pub id: PartId,
    /// do we allow multiple bodies fixed in relation to each other?
    /// Or is that a separate CompoundPartDefinition type?
    pub body: Body,
    /// there's interest in part metadata being exportable in STEP, etc, which means it must exist on engine.
    pub metadata: (),
}

#[derive(Clone)]
pub struct PartInstance {
    pub part_id: PartId,
    pub instance_id: PartInstanceId,
    pub body: EntityId,
    // anchors are assigned to body topologies but are referenceable from the part instance
    pub anchors: Vec<Anchor>,
}

#[derive(Clone)]
pub struct ModelingAnchor {
    pub id: EntityId,
}

#[derive(Clone)]
pub struct Anchor {
    pub id: EntityId,
    pub on: PartInstanceId,
    pub flip_z: bool,
}

#[derive(Clone)]
pub struct Mate {
    pub a: Anchor,
    pub a_target: PartInstanceId,
    pub b: Anchor,
    pub b_target: PartInstanceId,
}

#[derive(Clone)]
pub struct AssemblyDefinition {
    pub assembly_id: AssemblyId,
    pub part_instances: Vec<PartInstanceId>,
    pub instanced_subassemblies: Vec<AssemblyInstanceId>,
    /// though not solved yet, there may be n occurrences of the same imported definition,
    /// each with different mates, anchors, and relations applied to their components.
    pub imported_assembly_definitions: HashMap<AssemblyInstanceId, AssemblyDefinition>,
    pub anchors: Vec<Anchor>,
    pub mates: Vec<Mate>,
    pub poses: Vec<Pose>,
}

// a concrete, solved, assembly
#[derive(Clone)]
pub struct AssemblyInstance {
    pub assembly_id: AssemblyId,
    pub instance_id: AssemblyInstanceId,
    // keep anchors available for use from a parent assembly
    pub anchors: Vec<Anchor>,
    // components are instanced
    pub parts: Vec<PartInstanceId>,
    pub subassemblies: Vec<AssemblyInstanceId>,
    pub mates: Vec<Mate>,
    pub solve_result: Pose,
}

#[derive(Clone, Debug)]
pub struct PoseId(pub uuid::Uuid);

/// a Pose expresses a user space sparse set of transforms for the assembly.
/// The transforms are in the local space of the assembly definition.
/// It can be composed with another Pose.
/// ```
/// // pose1 is overridden by rhs (pose2)
/// myPose = pose1 + pose2
/// ```
/// The "pose chains" may be updated with values from a solve.
/// Which pose is updated if an instance all of them address has a new warm solve transform?
/// ```
/// myPose = pose1 + pose2 + pose3 + pose4 + pose5
/// ```
#[derive(Clone)]
pub struct Pose {
    pub id: PoseId,
    pub sparse_transforms: HashMap<PartInstanceId, Transform>,
    pub overrides: Option<Box<Pose>>, // the lhs Pose
}

/// The transform of a component within the local space of an assembly definition
#[derive(Clone, Debug, Default)]
pub struct Transform;
