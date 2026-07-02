import type { MascotFrame, MascotPose } from './types.js'

/** Hold a pose for n frames (60 ms each). */
function hold(pose: MascotPose, offset: number, frames: number): MascotFrame[] {
  return Array.from({ length: frames }, () => ({ pose, offset }))
}

// Offset 0 is normal and 1 is crouched. Renderers must keep a fixed-height
// container so the crouch clips instead of shifting surrounding content.
export const JUMP_WAVE: readonly MascotFrame[] = [
  ...hold('default', 1, 2),
  ...hold('arms-up', 0, 3),
  ...hold('default', 0, 1),
  ...hold('default', 1, 2),
  ...hold('arms-up', 0, 3),
  ...hold('default', 0, 1),
]

export const LOOK_AROUND: readonly MascotFrame[] = [
  ...hold('look-right', 0, 5),
  ...hold('look-left', 0, 5),
  ...hold('default', 0, 1),
]

export const CLICK_ANIMATIONS: readonly (readonly MascotFrame[])[] = [
  JUMP_WAVE,
  LOOK_AROUND,
]

export const IDLE_FRAME: MascotFrame = { pose: 'default', offset: 0 }
export const FRAME_MS = 60

export function chooseClickAnimation(random = Math.random): readonly MascotFrame[] {
  return CLICK_ANIMATIONS[Math.floor(random() * CLICK_ANIMATIONS.length)]!
}

