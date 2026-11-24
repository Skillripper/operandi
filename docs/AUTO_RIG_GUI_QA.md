# Auto Rig, milestone 1: GUI check

Auto Rig is based on Auto Character System by Lukasz Pazera (MIT).

For Jason or Codex (Claude's screen control can't click in Operandi). Use
the QA build from `scripts/bundle-dev.sh` with its own data folder. Each
step names the BEHAVIOR-ACS-FEATURES rule it checks.

Setup: a new scene. Import or model a character standing in a T pose,
facing +Z (front view looks at its face), its left arm towards +X. A
stack of boxes (body, two arms, two legs, head) is fine.

1. **New rig (ACS-10).** Rig tab ▸ Auto Rig ▸ **New Biped Rig**.
   A Biped item appears with a Guides group; small diamonds (green left,
   red right, blue centre) show a 1.8 m T-pose figure with dotted lines
   between them. Cmd+Z removes it all in one step; Shift+Cmd+Z brings it
   back.
2. **Fit (ACS-34).** Select the character mesh, Rig tab ▸ **Auto Rig ▸ Fit Guides
   to Mesh**. The guides jump to the character's size and sit inside its
   limbs, the head top on the top of the head, the hand guides at the
   fingertips, the toe guides at the toe tips. One undo step.
3. **Drag a guide (ACS-30, 31).** With no tool active, drag the left knee
   diamond in the front view. It follows the pointer, the right knee
   mirrors it live, and the shin and foot guides come along. Release: one
   undo step. Drag a centre guide sideways: it stays on the centre line.
4. **Symmetry off (ACS-31).** Auto Rig ▸ guides flyout ▸ Guide Symmetry. Drag the
   left wrist: the right one stays. **Mirror Guides Left to Right** puts it
   back in step.
5. **Build (ACS-32, 40).** Rig tab ▸ **Auto Rig ▸ Build Skeleton**. Joints appear on
   the guides (Hips, Spine1-3, Chest, Neck, Head, arms and legs), each
   bone pointing to the next joint. The guides stay visible (Guide
   context) and the joints don't show until step 6.
6. **Contexts (ACS-01, 02).** Auto Rig ▸ context flyout ▸ **Form Context**: guides hide, joints
   show, Setup turns on in the timeline. **Guide Context**: back to guides only.
   Switching is not in the undo history (Cmd+Z undoes the build, not the
   switch). Save, reopen: the context is the one you left.
7. **Options (ACS-22, 25).** In the options flyout, More Vertebrae and More
   Forearm Twist. The skeleton rebuilds with one more
   spine joint and one twist joint in each forearm. Fewer … takes them away.
8. **Bind (ACS-41).** Select the character mesh, Rig tab ▸ **Auto Rig ▸ Bind to Rig**. The rig goes to Form; the mesh is bound (a failed
   Heat bind selects the points to fix, as the Bind button does). Turn
   Setup off and rotate the Forearm_L joint: the forearm bends.
9. **Refit after binding (ACS-32).** Back in Guide, move the elbow guides,
   Build Skeleton again: the mesh doesn't jump in Setup, and still bends
   with the joints.
10. **No rig yet.** In a scene with no rig, Build Skeleton says "Add a
    Biped rig first (Rig tab ▸ New Biped Rig)".
