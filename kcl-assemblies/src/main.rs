use std::collections::HashMap;

use kcl_assemblies::domain::*;
use kcl_assemblies::domain_impl::*;

fn main() {
    // Parts take a function that lazily yields one or more Bodies.
    // omitting details of body creation for brevity
    let dog_body = KclPart::new(|| Body::new());
    let dog_upper_leg = KclPart::new(|| Body::new());
    let dog_lower_leg = KclPart::new(|| Body::new());
    let scene_brick = KclPart::new(|| Body::new());

    let some_unused_part = KclPart::new(|| Body::new()); // <-- this body construction never runs because it is not used by a program target
    let another_unused_part = KclPart::new(|| Body::new()); // <-- ditto

    // Assemblies take parts or other assemblies, and instance them into the definition.
    let dog_leg_assy = KclAssembly::new("first assy", || {
        let upper_leg = dog_upper_leg.definition().instance();
        let lower_leg = dog_lower_leg.definition().instance();

        // for brevity, omitting other anchor creation details for now
        let knee_upper_anchor = Anchor::from(&upper_leg);
        let knee_lower_anchor = Anchor::from(&lower_leg);
        let revolve_mate = Mate::new(&knee_upper_anchor, &knee_lower_anchor);

        let pose_extended_leg = Pose::new();
        let pose_bent_leg = Pose::new();

        // we'd like the block to handle this bit implicitly,
        // and let items be addressed by variable name rather than vec index
        AssemblyDefinition::new(
            vec![upper_leg.into(), lower_leg.into()],
            vec![knee_upper_anchor, knee_lower_anchor],
            vec![revolve_mate],
            vec![pose_extended_leg, pose_bent_leg],
        )
    });

    let dog = KclAssembly::new("second assy", || {
        // make the components of this subassembly concrete (execute required part modeling & subassemblies)
        // so that we have the user "input" for instancing.

        // make 4 copies of the same subassembly with the same pose.
        let dog_legs: Vec<AssemblyInstance<Solved>> = (0..3)
            .map(|_| {
                dog_leg_assy
                    .definition()
                    .with_pose(|poses| &poses[0])
                    .solved()
            })
            .collect();
        // make a copy of the concrete subassembly with a chosen base pose but solve it with the rest of the parent assembly.
        let moving_subassembly = dog_leg_assy.with_pose(|poses| &poses[1]);
        let torso = dog_body.definition().instance();

        let anchor = Anchor::new(&torso);
        // you can reference anchors from subassemblies - in KCL, by name.
        let fastened = Mate::new(
            &anchor,
            torso.instance_id,
            &dog_legs[0].anchors[1],
            dog_legs[0].instance_id,
        );

        // a Pose is intended to represent the transforms for influencing solve work not yet completed.
        // All components of subassemblies not yet solved might as well be, in solver & pose terms, direct components of this assembly.
        // So the Pose data for unsolved Subassemblies is stored in the parent assembly Pose.
        let pose = Pose::new();
        let other_pose = Pose::new();
        let third_pose = Pose::new();
        let mut leg_components: Vec<(InstanceId, ComponentInstance)> =
            dog_legs.iter().cloned().map(|leg| leg.into()).collect();
        leg_components.append(&mut vec![moving_subassembly.into(), torso.into()]);
        AssemblyBlockOutput::new(
            leg_components,
            vec![anchor],
            vec![fastened],
            vec![pose, other_pose, third_pose],
        )
    });

    // Scene block is invoked by the runtime if it was asked for this scene by name i.e. `zoo kcl main.kcl --views "my assembly"`.
    // Scene data is yielded in the KCL exec response.
    // We can execute multiple Scenes with overlapping dependencies without duplicating work.
    // Common case is multiple views of an assembly, or
    let scene = Scene::assembly("my assembly", || {
        let solved_assembly = dog.with_pose(|poses| &poses[0]).solved();

        // note that hide/show state is unique to a Scene.
        // You could have another Scene, same assembly, same Pose, same Solve, but with different visibility state.
        // in KCL we would want to reference these components by KCL instance name.
        hide(solved_assembly.components.get(&InstanceId::new()));
        hide(solved_assembly.components.get(&InstanceId::new()));
        // we can set up scene-specific section views as well when we support these.
        section_view();
        SceneInput
    });

    // Scene can yield Drawings, 3D View, or 2D Snapshots
}

// a view yields GUI data like gltf & Artifact Graph or similar
struct Scene<'a, T: Fn() -> SceneInput> {
    name: &'a str,
    program: T,
}

impl<'a, T: Fn() -> SceneInput> Scene<'a, T> {
    fn assembly(name: &'a str, program: T) -> Self {
        Self { name, program }
    }
}
fn hide(_instance_id: Option<&ComponentInstance>) {}
fn section_view() {}

// The Pose chain is flattened into a FlatPose - a single sparse map of InstanceId to transform - for solving.
// A complete map of InstanceId to transform is yielded by assembly solve.
// The original sparse pose chain is preserved so it can be updated post-solve for a warm solve next time.
#[derive(Clone)]
struct FlatPose {
    // we need to save the source Pose hierarchy because it gets edited with the solve outcome yielded by the FlatPose
    source: Pose,
    summed_sparse_transforms: HashMap<InstanceId, Transform>,
}

impl From<Pose> for FlatPose {
    fn from(value: Pose) -> Self {
        fn merge_sparse_transforms(
            lhs: HashMap<InstanceId, Transform>,
            mut rhs: HashMap<InstanceId, Transform>,
        ) -> HashMap<InstanceId, Transform> {
            for (instance_id, transform) in lhs {
                *rhs.entry(instance_id).or_default() += transform;
            }
            rhs
        }

        fn get_flat_transforms(pose: Pose) -> HashMap<InstanceId, Transform> {
            match pose.apply_onto {
                Some(lhs) => {
                    merge_sparse_transforms(get_flat_transforms(*lhs), pose.sparse_transforms)
                }
                None => pose.sparse_transforms,
            }
        }

        Self {
            source: value.clone(), // TODO: clone needed or can get_flat take a ref?
            summed_sparse_transforms: get_flat_transforms(value),
        }
    }
}

impl std::ops::Add for Transform {
    type Output = Transform;
    fn add(self, _rhs: Self) -> Self::Output {
        // not going to clutter this example with actual math
        Transform
    }
}

impl std::ops::AddAssign for Transform {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

#[derive(Clone, Debug)]
struct SolvedTransforms {
    instance_transforms: HashMap<InstanceId, Transform>,
}

impl SolvedTransforms {
    fn new() -> Self {
        Self {
            instance_transforms: HashMap::new(),
        }
    }
}

trait AssemblyInstanceState: Clone {}
#[derive(Clone)]
struct Unsolved {
    base_pose: Pose,
}
impl AssemblyInstanceState for Unsolved {}
#[derive(Clone)]
struct Solved {
    flat_pose: FlatPose,
    solved_transforms: SolvedTransforms,
}
impl AssemblyInstanceState for Solved {}

// a concrete assembly that has had a Pose applied
#[derive(Clone)]
struct AssemblyInstance<State: AssemblyInstanceState> {
    assembly_id: AssemblyId,
    instance_id: InstanceId,
    // keep anchors available for use from a parent assembly
    anchors: Vec<Anchor>,
    // components are instanced
    components: HashMap<InstanceId, ComponentInstance>,
    mates: Vec<Mate>,
    state: State,
}

impl AssemblyInstance<Unsolved> {
    fn solved(self) -> AssemblyInstance<Solved> {
        // we go to the engine and ask for a fresh instance of the assembly input data (components, anchors, mates)
        // we solve it with the user-supplied pose.
        let components: Vec<&ComponentInstance> = self.components.values().collect();
        let flat_pose: FlatPose = self.state.base_pose.into();
        let transforms =
            dummy_engine_solve_call(&components, &self.anchors, &self.mates, &flat_pose);
        AssemblyInstance {
            assembly_id: self.assembly_id,
            instance_id: self.instance_id,
            // anchors and components are unchanged by a solve.
            // transforms were what changed, but we don't store those on the anchors and components.
            anchors: self.anchors,
            components: self.components,
            mates: self.mates,
            state: Solved {
                solved_transforms: transforms,
                flat_pose,
            },
        }
    }
}

impl<T: AssemblyInstanceState> AssemblyInstance<T> {
    // when we duplicate what happens?
    // it's just a new reference to the source.
    fn duplicate(&self) -> Self {
        let instance_id = dummy_engine_duplicate(self.instance_id);
        AssemblyInstance {
            assembly_id: self.assembly_id,
            instance_id,
            anchors: self.anchors.clone(),
            components: self.components.clone(),
            mates: self.mates.clone(),
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
enum ComponentInstance {
    PartInstance(PartInstance),
    AssemblyInstanceSolved(AssemblyInstance<Solved>),
    AssemblyInstanceUnsolved(AssemblyInstance<Unsolved>),
}

impl From<PartInstance> for (InstanceId, ComponentInstance) {
    fn from(value: PartInstance) -> Self {
        (
            value.instance_id.clone(),
            ComponentInstance::PartInstance(value),
        )
    }
}

impl From<AssemblyInstance<Solved>> for (InstanceId, ComponentInstance) {
    fn from(value: AssemblyInstance<Solved>) -> Self {
        (
            value.instance_id.clone(),
            ComponentInstance::AssemblyInstanceSolved(value),
        )
    }
}

impl From<AssemblyInstance<Unsolved>> for (InstanceId, ComponentInstance) {
    fn from(value: AssemblyInstance<Unsolved>) -> Self {
        (
            value.instance_id.clone(),
            ComponentInstance::AssemblyInstanceUnsolved(value),
        )
    }
}

struct Bom {
    id: uuid::Uuid,
    parts: HashMap<PartId, u64>,
    children: HashMap<uuid::Uuid, Bom>,
}
