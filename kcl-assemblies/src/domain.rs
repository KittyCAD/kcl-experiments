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
pub struct EdgeSpecifier;

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
    /// the primitive anchors have no InstanceId yet associated with them
    pub primitive_anchors: Vec<PrimitiveAnchor>,
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

/// The primitive Anchor does not know about an InstanceId.
/// it was created during modeling operations.
/// It may be stored in a PartDefinition and converted to an Anchor
/// When that part is instanced.
#[derive(Clone)]
pub struct PrimitiveAnchor {
    pub id: EntityId,
    pub placement: AnchorPlacement,
}

/// the Anchor is mate-ready with both anchor and Instance IDs.
/// It can be created directly in an assembly block,
/// or derived from a primitiveAnchor stored in a PartDefinition when that part
/// is instanced and an InstanceId created to associate with.
#[derive(Clone)]
pub struct Anchor {
    pub id: EntityId,
    pub on: PartInstanceId,
    pub placement: AnchorPlacement,
}

/// if the server receives an AnchorPlacement it is unable to resolve,
/// it should return an error.
#[derive(Clone)]
pub struct AnchorPlacement {
    /// first compute location
    location: DerivedAnchorLocation,
    /// then compute orientation, which depends on location
    orientation: DerivedAnchorOrientation,
    /// then apply adjustments.
    /// We assume the user wants these to override the computed orientation.
    adjustments: AnchorAdjustment,
    /// the user may want to re-apply orientation after the location adjustments.
    reapply_orientation: bool,
    /// finally, you may want to flip the z axis.
    flip_z: bool,
}

#[derive(Clone)]
pub enum DerivedAnchorLocation {
    /// ideally these edge derivations apply to sketch edges as well
    EdgeMidpoint(EdgeSpecifier),
    EdgeStart(EdgeSpecifier),
    EdgeEnd(EdgeSpecifier),
    EdgeCircularCenter(EdgeSpecifier),
    /// intersection of the tangent directions of a circular arc's endpoints
    EdgeCircularVirtualCorner(EdgeSpecifier),
    FaceCentroid(EntityId),
    /// unsure what identifying information would be used to locate a sketch point
    SketchPoint(()),
    /// copy from existing Anchor
    Anchor(EntityId),
    Between {
        a: Box<DerivedAnchorLocation>,
        b: Box<DerivedAnchorLocation>,
    },
}

#[derive(Clone)]
pub enum DerivedAnchorOrientation {
    /// The face might not be planar.
    /// If not planar, sample edge normal at the closest point on face to AnchorLocation.
    FaceNormal(EntityId),
    SketchNormal(EntityId),
    /// The edge might not be a line.
    /// If not line, sample edge direction at the AnchorLocation
    EdgeDirection(EdgeSpecifier),
    /// copy from existing Anchor
    Anchor(EntityId),
    Average {
        a: Box<DerivedAnchorOrientation>,
        b: Box<DerivedAnchorOrientation>,
    },
}

#[derive(Clone)]
pub struct AnchorAdjustment {
    x: f64,
    y: f64,
    z: f64,
    /// These are probably exposed in KCL as an enum which maps to 90 deg increments.
    /// To avoid gimbal lock, don't permit rotations on y
    rotate_about_z: f64,
    rotate_about_x: f64,
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
