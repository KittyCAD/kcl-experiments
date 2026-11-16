use crate::domain::*;

pub fn define_part(from: Body) -> PartDefinition {
    PartDefinition {
        id: PartId::new(),
        body: from,
        metadata: (),
    }
}

pub fn instance_part(_part: &PartId) -> PartInstanceId {
    PartInstanceId::new()
}

pub fn define_assembly(_id: AssemblyId) -> AssemblyId {
    AssemblyId::new()
}

// instancing an assembly could yield `n` InstanceIds generated on engine. we just get the top level one.
// This is why Engine should own ID creation, single source of truth for IDs.
// Instances are solved.
pub fn instance_assembly(_assy: &AssemblyDefinition) -> AssemblyInstanceId {
    AssemblyInstanceId::new()
}

pub fn duplicate_part_instance() -> PartInstanceId {
    PartInstanceId::new()
}

/// The subcomponents of the new AssemblyInstance all have new globally unique IDs.
/// How do we want to sync those with KCL?
pub fn duplicate_assembly_instance() -> AssemblyInstanceId {
    AssemblyInstanceId::new()
}

/// The AssemblyInstanceId is sufficient information to solve,
/// because the engine already has a representation of the assembly definition.
///
/// The yielded Pose data should apply only to the top level assembly solved.
/// If the assembly contains subassemblies that were pre-solved, the pose data for those should not be included.
/// If the assembly contains imported assembly definitions, the pose data returned here applies to those.
pub fn solve(_assy: AssemblyInstanceId) -> Pose {
    Pose::new()
}
