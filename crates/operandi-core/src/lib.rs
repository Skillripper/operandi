//! Portable modeling state. No window, graphics, or platform API dependencies.
pub mod action_center;
pub mod bake;
pub mod brush_engine;
#[cfg(feature = "painting")]
#[path = "painting/paint.rs"]
pub mod paint;
#[cfg(feature = "painting")]
#[path = "painting/paint_geometry_brush.rs"]
pub mod paint_geometry_brush;
#[cfg(feature = "painting")]
#[path = "painting/paint_inks.rs"]
pub mod paint_inks;
#[cfg(feature = "painting")]
#[path = "painting/paint_lasso.rs"]
pub mod paint_lasso;
#[cfg(feature = "painting")]
#[path = "painting/paint_sculpt.rs"]
pub mod paint_sculpt;
#[cfg(feature = "painting")]
#[path = "painting/paint_starters.rs"]
pub mod paint_starters;
#[cfg(feature = "painting")]
#[path = "painting/paint_stroke.rs"]
pub mod paint_stroke;
#[cfg(feature = "painting")]
#[path = "painting/paint_tips.rs"]
pub mod paint_tips;
pub mod selection_inference;
pub mod uv_atlas;
pub mod uv_box;
pub mod uv_charts;
pub use action_center::*;
pub mod align_tools;
pub mod angle_snap;
#[path = "curve_tools/pen.rs"]
pub mod pen;
pub use align_tools::{
    GroundAlign, GroundMode, QuickAlign, QuickAlignKind, SelectionAxisMode, Tack, TackOptions,
};
#[path = "commands/clipboard.rs"]
pub mod clipboard;
#[path = "curve_tools/curve_boolean.rs"]
pub mod curve_boolean;
#[path = "curve_tools/curve_commands.rs"]
pub mod curve_commands;
#[path = "curve_tools/curve_convert.rs"]
pub mod curve_convert;
#[path = "curve_tools/curve_edit.rs"]
pub mod curve_edit;
#[path = "curve_tools/curve_fit.rs"]
pub mod curve_fit;
#[path = "curve_tools/curve_ops.rs"]
pub mod curve_ops;
#[path = "curve_tools/curves.rs"]
pub mod curves;
pub mod dicer;
#[path = "meshops/generator_ops.rs"]
pub mod generator_ops;
pub mod item_merge;
#[path = "curve_tools/patch.rs"]
pub mod patch;
#[path = "curve_tools/patch_live.rs"]
pub mod patch_live;
#[path = "curve_tools/path_tools.rs"]
pub mod path_tools;
#[path = "curve_tools/sketch.rs"]
pub mod sketch;
pub use clipboard::{Clipboard, PastePlacement};
#[path = "curve_tools/text.rs"]
pub mod text;
pub use curve_edit::{AddGeometry, CurveEdit};
pub use curves::CurveKind;
pub use text::{TextJustification, TextLocation, TextParameters};
#[path = "primitives/unit_insert.rs"]
pub mod unit_insert;
pub use pen::*;
pub use unit_insert::UnitShape;
pub mod background_constraint;
#[path = "render/camera_light.rs"]
pub mod camera_light;
#[path = "render/camera_light_defaults.rs"]
pub mod camera_light_defaults;
#[path = "render/camera_props.rs"]
pub mod camera_props;
mod corner_uv;
pub mod drop_action;
#[path = "render/environment.rs"]
pub mod environment;
#[path = "render/environment_channels.rs"]
mod environment_channels;
pub mod item_common;
pub mod item_common_channels;
pub mod item_delete;
pub mod item_extras;
pub mod item_groups;
pub mod item_list;
pub mod item_target;
pub mod item_transform_ops;
#[path = "render/locator.rs"]
pub mod locator;
#[path = "render/photometry.rs"]
pub mod photometry;
#[path = "render/physical_sun.rs"]
pub mod physical_sun;
pub use drop_action::DropAction;
#[path = "commands/command_record.rs"]
pub mod command_record;
pub mod editor;
pub use command_record::{CommandClass, CommandRecord, RecordLog, Source, StepRecords};
#[path = "commands/history_settings.rs"]
mod history_settings;
pub mod units;
#[path = "commands/user_defaults.rs"]
pub mod user_defaults;
pub use history_settings::*;
#[path = "polygons/flip.rs"]
mod flip;
#[path = "polygons/flip_it.rs"]
mod flip_it;
pub mod freeze;
#[path = "geometry/fxhash.rs"]
pub mod fxhash;
pub mod interchange;
pub mod lxo;
#[path = "geometry/mesh.rs"]
pub mod mesh;
pub mod multires;
#[path = "geometry/polygon.rs"]
pub mod polygon;
#[path = "render/render_settings.rs"]
pub mod render_settings;
#[path = "materials/shading.rs"]
pub mod shading;
pub mod subdivision;
pub use polygon::*;
#[path = "materials/blender_principled.rs"]
pub mod blender_principled;
#[path = "materials/cycles_noise.rs"]
pub mod cycles_noise;
#[path = "materials/cycles_voronoi.rs"]
pub mod cycles_voronoi;
#[path = "materials/dispersion.rs"]
pub mod dispersion;
#[path = "materials/gradient.rs"]
pub mod gradient;
#[path = "nodes/graph_view.rs"]
pub mod graph_view;
#[path = "render/light_path_expressions.rs"]
pub mod light_path_expressions;
#[path = "materials/material.rs"]
pub mod material;
#[path = "materials/material_values.rs"]
pub mod material_values;
pub mod materialx;
#[path = "nodes/node_graph.rs"]
pub mod node_graph;
#[path = "nodes/node_material.rs"]
pub mod node_material;
#[path = "nodes/node_view.rs"]
pub mod node_view;
#[path = "nodes/node_view_board.rs"]
pub mod node_view_board;
#[path = "nodes/node_view_commands.rs"]
pub mod node_view_commands;
#[path = "nodes/node_view_settings.rs"]
pub mod node_view_settings;
pub mod openpbr;
#[path = "render/per_light_outputs.rs"]
pub mod per_light_outputs;
pub mod planar_texture;
#[path = "meshops/procedural.rs"]
pub mod procedural;
#[path = "materials/process_occlusion.rs"]
pub mod process_occlusion;
#[path = "render/render_outputs.rs"]
pub mod render_outputs;
#[path = "materials/shader_eval.rs"]
pub mod shader_eval;
#[path = "materials/shader_func.rs"]
pub mod shader_func;
#[path = "materials/shader_instances.rs"]
pub mod shader_instances;
#[path = "materials/shader_layer_menu.rs"]
pub mod shader_layer_menu;
#[path = "materials/shader_library.rs"]
pub mod shader_library;
#[path = "materials/shader_tree.rs"]
pub mod shader_tree;
#[path = "materials/shader_wgsl.rs"]
pub mod shader_wgsl;
#[path = "materials/texture_locator.rs"]
pub mod texture_locator;
#[path = "anim/transform_channel.rs"]
pub mod transform_channel;
#[path = "polygons/vertex_map_texture.rs"]
pub mod vertex_map_texture;
pub use material::{Material, MaterialKind};
pub use shader_tree::ShaderTreeEdit;
#[path = "polygons/polygon_material.rs"]
pub mod polygon_material;
pub use polygon_material::*;
#[path = "polygons/flatten_ngons.rs"]
mod flatten_ngons;
#[path = "polygons/polygon_align.rs"]
mod polygon_align;
pub use flatten_ngons::*;
#[path = "polygons/polygon_part.rs"]
pub mod polygon_part;
#[path = "polygons/polygon_tags.rs"]
pub mod polygon_tags;
pub use polygon_tags::*;
#[path = "materials/colour_math.rs"]
pub mod colour_math;
#[path = "curve_tools/contour.rs"]
pub mod contour;
#[path = "edges/diagonal_redirect.rs"]
pub mod diagonal_redirect;
#[path = "edges/edge_zip.rs"]
pub mod edge_zip;
#[path = "materials/editor_color.rs"]
pub mod editor_color;
#[path = "polygons/fix_black_dots.rs"]
pub mod fix_black_dots;
#[path = "duplication/merge_instance.rs"]
pub mod merge_instance;
#[path = "painting/mesh_paint.rs"]
pub mod mesh_paint;
#[path = "primitives/plane_item.rs"]
pub mod plane_item;
pub mod scaffold;
#[path = "primitives/teapot.rs"]
pub mod teapot;
pub use editor_color::EditorColor;
pub mod tag_selection;
pub use tag_selection::{TagRegion, TagSelection};
#[path = "polygons/polygon_quadruple.rs"]
mod polygon_quadruple;
#[path = "polygons/polygon_scale_offset.rs"]
mod polygon_scale_offset;
#[path = "polygons/polygon_spin.rs"]
mod polygon_spin;
#[path = "polygons/polygon_split.rs"]
mod polygon_split;
#[path = "polygons/polygon_unify.rs"]
pub mod polygon_unify;
pub use polygon_part::*;
pub use polygon_unify::*;
#[path = "edges/bridge_commands.rs"]
mod bridge_commands;
#[path = "anim/channels.rs"]
pub mod channels;
#[path = "rigging/character_rig.rs"]
pub mod character_rig;
#[path = "rigging/character_rig_commands.rs"]
pub mod character_rig_commands;
#[path = "commands/command_args.rs"]
pub mod command_args;
#[path = "commands/command_exec.rs"]
pub mod command_exec;
#[path = "commands/command_line.rs"]
pub mod command_line;
#[path = "commands/command_names.rs"]
pub mod command_names;
pub mod command_registry;
#[path = "commands/command_runtime.rs"]
pub mod command_runtime;
mod set_position;
pub use set_position::*;
mod absolute_scaling;
pub use absolute_scaling::*;
#[path = "polygons/polygon_subdivide.rs"]
pub mod polygon_subdivide;
pub use polygon_subdivide::{
    SubdivideBoundary, SubdivideMethod, SubdivideOptions, SubdivideOutcome, UnsubdivideOptions,
};
#[path = "polygons/polygon_triple.rs"]
pub mod polygon_triple;
pub use polygon_triple::*;
#[path = "polygons/mesh_cleanup.rs"]
pub mod mesh_cleanup;
pub use mesh_cleanup::*;
#[path = "polygons/polygon_remove.rs"]
pub mod polygon_remove;
pub use polygon_remove::*;
#[path = "polygons/polygon_merge.rs"]
pub mod polygon_merge;
pub use polygon_merge::*;
#[path = "polygons/polygon_collapse.rs"]
pub mod polygon_collapse;
pub use polygon_collapse::*;
#[path = "edges/edge_extend.rs"]
pub mod edge_extend;
pub use edge_extend::{
    EXTEND_UV_MAP, EdgeExtendOptions, ExtendHandles, ExtendHaul, ExtendPlane, ExtendShape,
    ExtendUvs, edge_extend_at_zero, edge_extend_geometry, edge_extend_shaped,
};
#[path = "edges/edge_extrude.rs"]
pub mod edge_extrude;
pub use edge_extrude::{
    edge_extrude_geometry, edge_extrude_start_width, edge_extrude_with, vertex_extrude_start_width,
};
#[path = "edges/edge_slide.rs"]
pub mod edge_slide;
pub use edge_slide::{
    SlideExtras, edge_slide_extra, edge_slide_geometry, edge_slide_weighted, edge_slide_with,
};
#[path = "edges/edge_crease.rs"]
pub mod edge_crease;
pub use edge_crease::edge_crease_geometry;
#[path = "edges/edge_bevel.rs"]
pub mod edge_bevel;
#[path = "edges/edge_bevel_junctions.rs"]
mod edge_bevel_junctions;
pub use edge_bevel::{
    EdgeBevelMode, EdgeBevelOptions, EdgeBevelPreset, EdgeBevelResult, EdgeBevelSettings,
    EdgeBevelShape, EdgeEnd, edge_bevel_geometry, edge_bevel_with,
};
#[path = "edges/edge_commands.rs"]
pub mod edge_commands;
#[path = "edges/edge_relax.rs"]
pub mod edge_relax;
pub use edge_commands::{
    EdgeCommand, edge_bridge, edge_bridge_geometry, edge_command_geometry, edge_delete_geometry,
    edge_join_symmetric_geometry, edge_subdivide_geometry,
};
pub use edge_relax::edge_relax_geometry;
pub mod axis_slice;
#[path = "polygons/detriangulate.rs"]
pub mod detriangulate;
pub mod uv_edit;
pub mod uv_export;
pub mod uv_maps;
pub mod uv_ops;
pub mod uv_peel;
pub mod uv_pipe;
pub mod uv_project;
pub mod uv_relax;
pub mod uv_seams;
pub mod uv_symmetry;
pub mod uv_tools;
pub use detriangulate::DetriangulateOptions;
#[path = "anim/anim_camera_channels.rs"]
pub mod anim_camera_channels;
#[path = "anim/anim_commands.rs"]
pub mod anim_commands;
#[path = "anim/anim_curve.rs"]
pub mod anim_curve;
#[path = "anim/anim_edit.rs"]
pub mod anim_edit;
#[path = "anim/anim_graph.rs"]
pub mod anim_graph;
#[path = "anim/animation.rs"]
pub mod animation;
pub mod boolean;
#[path = "anim/channel_sets.rs"]
pub mod channel_sets;
pub mod click_select;
pub mod drill;
#[path = "meshops/eval_build.rs"]
pub mod eval_build;
#[path = "meshops/eval_graph.rs"]
pub mod eval_graph;
#[path = "meshops/eval_state.rs"]
pub mod eval_state;
#[path = "meshops/evaluate.rs"]
pub mod evaluate;
#[path = "rigging/ik.rs"]
pub mod ik;
#[path = "rigging/ik_body.rs"]
pub mod ik_body;
#[path = "rigging/ik_commands.rs"]
pub mod ik_commands;
#[path = "rigging/ik_joint.rs"]
pub mod ik_joint;
#[path = "rigging/ik_switch.rs"]
pub mod ik_switch;
pub mod kit_item;
#[path = "meshops/mesh_stack.rs"]
pub mod mesh_stack;
#[path = "meshops/mesh_stack_builtin.rs"]
mod mesh_stack_builtin;
#[path = "meshops/mesh_stack_editor.rs"]
pub mod mesh_stack_editor;
#[path = "meshops/mesh_stack_items.rs"]
pub mod mesh_stack_items;
#[path = "meshops/mesh_stack_tools.rs"]
pub mod mesh_stack_tools;
pub mod meshblend;
pub mod meshblend_spike;
#[path = "meshops/meshop_selection.rs"]
pub mod meshop_selection;
#[path = "meshops/meshop_types.rs"]
pub mod meshop_types;
#[path = "rigging/pose_tool.rs"]
pub mod pose_tool;
#[path = "rigging/rig.rs"]
pub mod rig;
#[path = "rigging/rig_commands.rs"]
pub mod rig_commands;
#[path = "rigging/rig_eval.rs"]
pub mod rig_eval;
#[path = "rigging/rig_expression.rs"]
pub mod rig_expression;
#[path = "rigging/rig_list.rs"]
pub mod rig_list;
#[path = "rigging/rig_schematic.rs"]
pub mod rig_schematic;
#[path = "rigging/rig_user_set.rs"]
pub mod rig_user_set;
pub mod scene_time;
#[path = "meshops/script_mesh.rs"]
pub mod script_mesh;
#[path = "rigging/skeleton.rs"]
pub mod skeleton;
#[path = "rigging/skin.rs"]
pub mod skin;
#[path = "rigging/skin_bind.rs"]
pub mod skin_bind;
#[path = "rigging/skin_commands.rs"]
pub mod skin_commands;
#[path = "rigging/skin_weights.rs"]
pub mod skin_weights;
#[path = "anim/time_tick.rs"]
pub mod time_tick;
pub use boolean::{BooleanOp, BooleanOptions, BooleanProblem, DriveMesh};
pub use drill::{DrillAxis, DrillKind, DrillOp, DrillOptions};
#[path = "duplication/duplicate.rs"]
pub mod duplicate;
#[path = "duplication/duplicate_copier.rs"]
pub mod duplicate_copier;
#[path = "duplication/duplicate_hierarchy.rs"]
mod duplicate_hierarchy;
#[path = "duplication/duplicate_items.rs"]
pub mod duplicate_items;
#[path = "duplication/duplicate_sweep.rs"]
pub mod duplicate_sweep;
#[path = "duplication/duplicate_transform.rs"]
pub mod duplicate_transform;
pub use duplicate::{
    Array, Circle, CloneLine, Copies, DuplicateParameters, Lathe, Mirror, MirrorMode, RadialArray,
    Sweep, UvsAround,
};
#[path = "edges/edge_insert.rs"]
pub mod edge_insert;
pub mod path_slice;
pub use axis_slice::{AxisSlice, AxisSliceMode, axis_slice_bounds, axis_slice_planes};
pub mod profile;
pub use profile::{ProfilePick, ProfilePoint, ProfileShape};
pub mod slice;
pub(crate) mod slice_cut;
pub use edge_insert::{
    EdgeInsertOptions, add_loop, add_loop_ring, add_loop_with, add_point, add_point_with,
};
pub use path_slice::{
    CurveSlice, CutPath, PathCutResult, PathCutter, PenSlice, PrimitiveShape, PrimitiveSlice,
    ShapePlace, SliceAxis, SliceOperation, path_cut,
};
pub use slice::{
    EdgeSlice, LoopSlice, LoopSliceEdit, LoopSliceMode, OffsetSide, PlaneCut, PolygonSlice,
    SliceParameters, SlicePoint, SliceShow,
};
#[path = "polygons/polygon_make.rs"]
pub mod polygon_make;
#[path = "polygons/vertex_commands.rs"]
pub mod vertex_commands;
pub use polygon_make::*;
pub use vertex_commands::{
    MakeType, MergeOptions, ReorderMode, VERTEX_BEVEL_MAX_ROUND_LEVEL, VertexCommand, WeldDrop,
    add_points, make_polygon_from_points, make_polygon_typed, sync_index_order,
    vertex_bevel_geometry, vertex_bevel_round, vertex_command_geometry, vertex_extrude_geometry,
    vertex_extrude_with, vertex_join_symmetric_geometry, vertex_merge_geometry, vertex_merge_with,
    vertex_slide_direction, vertex_slide_geometry, vertex_tear_geometry, weld_drag_geometry,
    weld_points,
};
pub mod numeric;
#[path = "deformers/tweak.rs"]
pub mod tweak;
pub use tweak::{
    TweakConnect, TweakElement, TweakGrab, TweakMirror, TweakMode, TweakOptions, TweakPlan,
    edge_loop_points, tweak_apply, tweak_mirror, tweak_mirror_points, tweak_plan, tweak_points,
    tweak_points_mirrored,
};
#[path = "geometry/mesh_pack.rs"]
pub mod mesh_pack;
#[path = "geometry/mesh_slot.rs"]
pub mod mesh_slot;
#[path = "nodes/node_graph_reader.rs"]
pub mod node_graph_reader;
#[path = "geometry/packed_mesh.rs"]
pub mod packed_mesh;
pub use mesh_slot::{MeshSlot, MeshView};
#[path = "duplication/mesh_instance.rs"]
pub mod mesh_instance;
pub use packed_mesh::{MeshRead, PackedMesh};
#[path = "commands/document_state.rs"]
mod document_state;
#[path = "commands/persistence.rs"]
pub mod persistence;
pub mod preset;
#[path = "primitives/primitive.rs"]
pub mod primitive;
pub mod transform_repeat;
#[path = "commands/undo_memory.rs"]
mod undo_memory;
pub use primitive::*;
pub mod primitive_uv;
pub mod project;
pub mod project_paths;
pub use primitive_uv::*;
pub use project::Project;
#[path = "duplication/radial.rs"]
pub mod radial;
pub use radial::*;
pub mod active_task;
#[path = "render/backdrop.rs"]
pub mod backdrop;
pub mod last_mesh;
pub mod multi_active;
pub mod multi_tools;
pub mod multi_transform;
pub use multi_active::ComponentSelection;
pub mod scene;
pub use active_task::ActiveTask;
pub mod selection;
pub mod selection_sets;
pub use selection_sets::{SetKind, SetRef, UseSetMode};
mod selection_continuation;
mod selection_neighbour_loop;
mod selection_symmetry;
pub use editor::*;
pub use mesh::*;
pub use scene::*;
pub use selection::*;
#[path = "deformers/bend.rs"]
pub mod bend;
pub mod click_start;
#[path = "deformers/deform.rs"]
pub mod deform;
#[path = "deformers/deform_finish.rs"]
pub mod deform_finish;
#[path = "deformers/falloff.rs"]
pub mod falloff;
#[path = "meshops/mesh_stack_deform.rs"]
pub mod mesh_stack_deform;
pub mod select_tools;
#[path = "geometry/topology_cleanup.rs"]
pub mod topology_cleanup;
#[path = "curve_tools/topology_pen.rs"]
pub mod topology_pen;
pub use bend::BendOptions;
pub use deform::{DeformPreset, PushNormals, SmoothOptions};
pub use deform_finish::{
    JitterOptions, LinearAlignOptions, LinearMode, QuantizeOptions, RadialAlignOptions, RadialMode,
    RelaxMeshOptions,
};
pub use topology_pen::{PenEdit, PenMode, TopologyPenOptions};
pub mod haul;
pub mod pivot_service;
pub mod tool_preset;
mod tool_reset;
pub use tool_preset::{PresetRow, PresetSession};
pub mod scene_index;
#[path = "geometry/shared.rs"]
pub mod shared;
#[path = "geometry/topology.rs"]
pub mod topology;
#[path = "geometry/topology_cache.rs"]
pub mod topology_cache;
pub use falloff::{
    ActiveFalloff, CylinderFalloff, Falloff, FalloffShape, FalloffSymmetric, LinearFalloff,
    MixMode, RadialFalloff, SelectionFalloff, VertexMapFalloff, VertexMapMode,
};
#[path = "deformers/falloff_path.rs"]
pub mod falloff_path;
#[path = "deformers/falloff_pattern.rs"]
pub mod falloff_pattern;
pub use falloff_path::{CurvatureFalloff, PathFalloff};
#[path = "deformers/falloff_reach.rs"]
pub mod falloff_reach;
#[path = "deformers/falloff_screen.rs"]
pub mod falloff_screen;
pub use falloff_pattern::{GrayImage, ImageFalloff, NoiseFalloff};
pub use falloff_reach::{CoplanarFalloff, ElementFalloff, SoftSelectionFalloff};
pub use falloff_screen::{AirbrushFalloff, LassoFalloff, LassoStyle, ScreenFalloff};
pub mod sculpt;
/// Blender-derived brush maths (GPL-2.0-or-later, see the module header).
pub mod sculpt_blender;
// The rest stays under `sculpt::` (its `Stroke` would clash with egui's in
// the app's glob imports).
pub use sculpt::{SculptBrush, SculptEdit, SculptItemEdit, SculptSettings};
pub mod symmetrize_tool;
pub mod symmetry;
pub mod symmetry_fix;
pub mod symmetry_sides;
pub mod symmetry_support;
pub mod tool_card;
pub use shared::Shared;
pub use symmetry::{Side, SymmetrizeSide, Symmetry};
pub use tool_card::ToolCard;
pub type Result<T> = std::result::Result<T, String>;

#[path = "primitives/cone.rs"]
pub mod cone;
pub use cone::*;
#[path = "polygons/inset.rs"]
pub mod inset;
#[path = "polygons/mesh_cleanup_ops.rs"]
pub mod mesh_cleanup_ops;
pub use inset::{InsetResult, inset_geometry, shift_geometry};
pub use mesh_cleanup_ops::{CleanupOption, CleanupResult, mesh_cleanup_geometry};
#[path = "geometry/mesh_attributes.rs"]
pub mod mesh_attributes;
pub use mesh_attributes::*;

#[path = "primitives/cone_editor.rs"]
pub mod cone_editor;
mod numeric_session;
pub use numeric_session::*;

#[path = "primitives/cone_grid.rs"]
mod cone_grid;

#[path = "primitives/ellipsoid.rs"]
pub mod ellipsoid;
pub use ellipsoid::*;
#[path = "primitives/ellipsoid_editor.rs"]
mod ellipsoid_editor;

#[path = "primitives/shapes.rs"]
pub mod shapes;
pub use shapes::*;
pub mod element_state;
pub use element_state::{ElementKind, ElementMask, ElementOp, ElementState, Protection};
#[path = "primitives/shape_editor.rs"]
mod shape_editor;

#[path = "polygons/spike.rs"]
pub mod spike;
pub use spike::*;
#[path = "polygons/spike_editor.rs"]
mod spike_editor;

#[path = "polygons/extrude.rs"]
pub mod extrude;
pub use extrude::*;

#[path = "polygons/smooth_shift.rs"]
pub mod smooth_shift;
pub use smooth_shift::*;

#[path = "polygons/smooth_shift_editor.rs"]
mod smooth_shift_editor;

#[path = "polygons/bevel.rs"]
pub mod bevel;
pub use bevel::*;

#[path = "polygons/twist_bevel.rs"]
pub mod twist_bevel;
pub use twist_bevel::{
    TubeFrame, TwistAxis, TwistBevelOptions, TwistBevelPipe, TwistBevelResult, twist_bevel_frames,
    twist_bevel_geometry, twist_bevel_rings,
};

#[path = "polygons/bevel_editor.rs"]
mod bevel_editor;
pub use bevel_editor::*;

#[path = "polygons/bevel_profile.rs"]
mod bevel_profile;
pub use bevel_profile::*;

#[path = "polygons/bevel_weld.rs"]
mod bevel_weld;

#[path = "polygons/correspondence.rs"]
pub mod correspondence;
pub use correspondence::*;
pub mod identity;
pub use identity::*;

mod identity_editor;

#[path = "polygons/live_extrude.rs"]
pub mod live_extrude;
pub use live_extrude::*;

#[path = "commands/history_policy.rs"]
mod history_policy;
pub(crate) mod mid_tool_undo;
pub use history_policy::*;
pub use mid_tool_undo::{ToolReopen, ToolTweaks};

#[path = "commands/session_ownership.rs"]
mod session_ownership;

#[path = "commands/history_completion.rs"]
mod history_completion;
pub use history_completion::*;

mod mode_eligibility;
pub use mode_eligibility::*;

#[path = "edges/bridge.rs"]
pub mod bridge;
pub use bridge::*;
#[path = "edges/bridge_editor.rs"]
mod bridge_editor;

#[path = "curve_tools/pen_interactive.rs"]
pub mod pen_interactive;
pub use pen_interactive::*;
pub mod guides;
#[path = "polygons/reduction.rs"]
pub mod reduction;
pub mod remesh;
pub mod remesh_voxel;
#[path = "duplication/replicator.rs"]
pub mod replicator;
#[path = "polygons/vertex_normals.rs"]
pub mod vertex_normals;
pub use remesh::{CountMode, OctreeMode, OctreeOptions, RemeshOp, RetopoOptions, VoxelOptions};
#[path = "polygons/reduction_qem.rs"]
mod reduction_qem;
pub use reduction::*;
#[path = "polygons/reduction_editor.rs"]
mod reduction_editor;
pub use reduction_editor::*;

#[path = "edges/chamfer.rs"]
pub mod chamfer;
pub use chamfer::*;

#[path = "deformers/poly_haul.rs"]
pub mod poly_haul;
pub use poly_haul::*;
#[path = "deformers/poly_haul_modes.rs"]
pub mod poly_haul_modes;
pub use poly_haul_modes::{HaulAmount, HaulTarget, PolyHaulMode, poly_haul_mode_geometry};

#[path = "curve_tools/sketch_extrude.rs"]
pub mod sketch_extrude;
pub mod solid_sketch;
#[path = "curve_tools/solid_sketch_editor.rs"]
pub mod solid_sketch_editor;
pub use solid_sketch_editor::{SketchEdit, SketchSession};
#[path = "curve_tools/solid_sketch_store.rs"]
pub mod solid_sketch_store;
#[path = "curve_tools/solid_sketch_symmetry.rs"]
pub mod solid_sketch_symmetry;
pub mod statistics;
pub mod statistics_edit;
pub use sketch_extrude::*;
