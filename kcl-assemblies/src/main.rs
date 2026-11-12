use std::{cell::OnceCell, collections::HashMap};

fn main() {
    // Parts take a function that lazily yields one or more Bodies.
    let part = Part::new(|| Body::new());
    let part2 = Part::new(|| Body::new()); // <-- this body construction never runs because it is not used by a program target
    let part3 = Part::new(|| Body::new()); // <-- ditto
    let assy = AssemblyDefinition::new("first assy", || {
        let instance = part.instance();
        let instance2 = part.instance();

        let a = Anchor::new(&instance);
        let b = Anchor::new(&instance2);
        let mate = Mate::new(&a, &b);

        let pose = Pose::new();

        // we'd like the block to handle this bit implicitly,
        // and let items be addressed by variable name rather than vec index
        AssemblyInput::new(
            vec![instance.into(), instance2.into()],
            vec![a, b],
            vec![mate],
            vec![pose],
        )
    });

    let assy2 = AssemblyDefinition::new("second assy", || {
        // make the components of this subassembly concrete (execute required part modeling & subassemblies)
        // so that we have the user "input" for instancing.

        // make a copy of the concrete subassembly and solve it with this pose
        let frozen_subassembly = assy.with_pose(|poses| &poses[0]).solved();
        // make a copy of the concrete subassembly with a chosen base pose but solve it with the rest of the parent assembly.
        let moving_subassembly = assy.with_pose(|poses| &poses[1]);
        let some_part = part.instance();

        let anchor = Anchor::new(&some_part);
        // you can reference anchors from subassemblies - in KCL, by name.
        let fastened = Mate::new(&anchor, &frozen_subassembly.anchors[1]);

        // a Pose is intended to represent the transforms for influencing solve work not yet completed.
        // All components of subassemblies not yet solved might as well be, in solver & pose terms, direct components of this assembly.
        // So the Pose data for unsolved Subassemblies is stored in the parent assembly Pose.
        let pose = Pose::new();
        let other_pose = Pose::new();
        let third_pose = Pose::new();
        AssemblyInput::new(
            vec![
                moving_subassembly.into(),
                frozen_subassembly.into(),
                some_part.into(),
            ],
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
        let solved_assembly = assy2.with_pose(|poses| &poses[0]).solved();

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

struct SceneInput;

// a ReleasePackage yields all invoked data
struct ReleasePackage;

impl ReleasePackage {
    fn new<F>(_name: &str, workflow: F) -> Self
    where
        F: Fn(),
    {
        workflow();
        Self
    }
}

// thinking in terms of release management
// set rules on part and assembly definitions
// set rules for views? probably not needed.

// abstract part design
#[derive(Copy, Clone, Debug)]
struct PartId(uuid::Uuid);
impl PartId {
    fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
// abstract assembly that can yield many concrete poses
#[derive(Copy, Clone, Debug)]
struct AssemblyId(uuid::Uuid);

impl AssemblyId {
    fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
// concrete assembly or part instances
// no Copy impl because it shouldn't be common to reuse the same ID in a new place.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct InstanceId(uuid::Uuid);

impl InstanceId {
    fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

#[derive(Clone, Copy, Debug)]
struct BodyId(uuid::Uuid);

impl BodyId {
    fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

#[derive(Clone)]
struct Body {
    id: BodyId,
}

impl Body {
    fn new() -> Self {
        Self { id: BodyId::new() }
    }
}

struct Part<Constructor: Fn() -> Body> {
    part_id: PartId,
    constructor: Constructor,
    body: OnceCell<Body>,
}

impl<Constructor: Fn() -> Body> Part<Constructor> {
    fn new(constructor: Constructor) -> Self {
        Self {
            part_id: PartId::new(),
            constructor,
            body: OnceCell::new(),
        }
    }

    fn instance(&self) -> PartInstance {
        // A given Part should only execute its topology one time for all instances.
        let body = self.body.get_or_init(|| {
            let body = (self.constructor)();
            dummy_engine_convert_body_to_part_definition(self.part_id, body.id);
            body
        });
        // The body topology exists: now make an instance of it and yield an ID back to KCL.
        let instance_id = dummy_engine_make_part_instance(self.part_id);
        PartInstance {
            part_id: self.part_id,
            instance_id,
            body: body.clone(),
            anchors: Vec::new(),
        }
    }
}

// Tell engine to take the body "offscreen", put it somewhere, none of our business where.
fn dummy_engine_convert_body_to_part_definition(part_id: PartId, body_id: BodyId) {}

fn dummy_engine_make_part_instance(part_id: PartId) -> InstanceId {
    InstanceId::new()
}

#[derive(Clone)]
struct PartInstance {
    part_id: PartId,
    instance_id: InstanceId,
    body: Body,
    anchors: Vec<Anchor>,
}

#[derive(Clone)]
struct AssemblyDefinition<'a, Constructor: Fn() -> AssemblyInput> {
    assembly_id: AssemblyId,
    name: &'a str,
    constructor: Constructor,
    inputs: OnceCell<AssemblyInput>,
}

impl<'a, Constructor: Fn() -> AssemblyInput> AssemblyDefinition<'a, Constructor> {
    fn new(name: &'a str, constructor: Constructor) -> AssemblyDefinition<'a, Constructor> {
        AssemblyDefinition {
            assembly_id: AssemblyId::new(),
            name,
            constructor,
            inputs: OnceCell::new(),
        }
    }

    fn with_pose<F>(&self, pose_fn: F) -> AssemblyInstance<Unsolved>
    where
        F: Fn(&[Pose]) -> &Pose,
    {
        let input = self.inputs.get_or_init(|| (self.constructor)());
        let pose = pose_fn(&input.poses);
        // we go to the engine and ask for fresh instances of the assembly input data (components, anchors, mates)
        let (components, anchors) = dummy_engine_make_unsolved_instances(&input);
        let components = components
            .into_iter()
            .collect::<HashMap<InstanceId, ComponentInstance>>();
        AssemblyInstance {
            assembly_id: self.assembly_id,
            instance_id: InstanceId::new(),
            anchors,
            components,
            mates: input.mates.clone(),
            state: Unsolved {
                base_pose: pose.clone(),
            },
        }
    }

    // A given assembly definition should only be solved once for each unique Pose.
    // When that solve happens, we go find the Pose and give it the outcome for a warm solve next time.
    // if the Pose was a composite of multiple Poses, we try to alter the higher level poses first if entries for the instances exist there.
    // We do not delve into the subassembly definition and mutate Pose data within that subassembly.
    fn update_pose_for_warm_solve(&mut self, solve_outcome: Solved) {
        // go through every Pose cited in the final_pose and update their transforms to match the solve outcome.
        // we could start with the Flat Pose and map back from Flat to Pose. or start with Pose. but how would we know which thing to map to?
        let Solved {
            flat_pose:
                FlatPose {
                    source,
                    summed_sparse_transforms: sparse_transforms,
                },
            solved_transforms,
        } = solve_outcome;
    }
}

#[derive(Clone)]
struct AssemblyInput {
    components: Vec<(InstanceId, ComponentInstance)>,
    anchors: Vec<Anchor>,
    mates: Vec<Mate>,
    poses: Vec<Pose>,
}

impl AssemblyInput {
    fn new(
        components: Vec<(InstanceId, ComponentInstance)>,
        anchors: Vec<Anchor>,
        mates: Vec<Mate>,
        poses: Vec<Pose>,
    ) -> Self {
        Self {
            components,
            anchors,
            mates,
            poses,
        }
    }
}

// a Pose expresses a user space sparse set of transforms for the assembly.
// It can be added with another Pose.
#[derive(Clone)]
struct Pose {
    id: PoseId,
    sparse_transforms: HashMap<InstanceId, Transform>,
    apply_onto: Option<Box<Pose>>, // the lhs Pose
}

impl Pose {
    fn new() -> Self {
        Self {
            id: PoseId::new(),
            sparse_transforms: HashMap::new(),
            apply_onto: None,
        }
    }
}

impl std::ops::Add for Pose {
    type Output = Self;
    fn add(self, mut rhs: Self) -> Self::Output {
        // note that new_pose _is_ my_sparse_adjustment but it now knows to be applied onto existing_pose
        // new_pose = existing_pose + my_sparse_adjustment
        rhs.apply_onto = Some(Box::new(self));
        rhs
    }
}

impl std::ops::AddAssign for Pose {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

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

#[derive(Clone, Debug)]
struct PoseId(uuid::Uuid);
impl PoseId {
    fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

#[derive(Clone, Debug, Default)]
struct Transform;

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

trait AssemblyInstanceState {}
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

fn dummy_engine_solve_call(
    _components: &[&ComponentInstance],
    _anchors: &[Anchor],
    _mates: &[Mate],
    _pose: &FlatPose,
) -> SolvedTransforms {
    // dummy body
    SolvedTransforms::new()
}

fn dummy_engine_make_unsolved_instances(
    assembly_input: &AssemblyInput,
) -> (Vec<(InstanceId, ComponentInstance)>, Vec<Anchor>) {
    (
        assembly_input.components.clone(),
        assembly_input.anchors.clone(),
    )
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

#[derive(Clone)]
struct Mate {
    a: Anchor,
    b: Anchor,
}
impl Mate {
    fn new(a: &Anchor, b: &Anchor) -> Self {
        Self {
            a: a.clone(),
            b: b.clone(),
        }
    }
}

#[derive(Clone)]
struct Anchor;

impl Anchor {
    fn new(_on: &PartInstance) -> Self {
        Self
    }
}

struct Bom {
    id: uuid::Uuid,
    parts: HashMap<PartId, u64>,
    children: HashMap<uuid::Uuid, Bom>,
}
