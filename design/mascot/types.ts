export type MascotPose = 'default' | 'arms-up' | 'look-left' | 'look-right'

export type MascotSize = 'small' | 'large'

export type SmallSegments = {
  r0: string
  r1L: string
  r1E: string
  r1R: string
  r2L: string
  r2M: string
  r2R: string
  r3: string
}

export type MascotFrame = {
  pose: MascotPose
  offset: number
}

