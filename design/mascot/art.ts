import type { MascotPose, SmallSegments } from './types.js'

export const LARGE_ART: Record<MascotPose, string[]> = {
  default: [
    '           ▄▄           ',
    '     ▗▄▄▄▄▄██▄▄▄▄▄▖     ',
    '     ▐ ────────── ▌     ',
    '     ▐   ●    ●   ▌     ',
    '     ▐    ╰──╯    ▌     ',
    '     ▝▀▀▀▀▀▀▀▀▀▀▀▀▘     ',
    '       ▝▘      ▝▘       ',
  ],
  'look-left': [
    '           ▄▄           ',
    '     ▗▄▄▄▄▄██▄▄▄▄▄▖     ',
    '     ▐ ────────── ▌     ',
    '     ▐ ●    ●     ▌     ',
    '     ▐    ╰──╯    ▌     ',
    '     ▝▀▀▀▀▀▀▀▀▀▀▀▀▘     ',
    '       ▝▘      ▝▘       ',
  ],
  'look-right': [
    '           ▄▄           ',
    '     ▗▄▄▄▄▄██▄▄▄▄▄▖     ',
    '     ▐ ────────── ▌     ',
    '     ▐     ●    ● ▌     ',
    '     ▐    ╰──╯    ▌     ',
    '     ▝▀▀▀▀▀▀▀▀▀▀▀▀▘     ',
    '       ▝▘      ▝▘       ',
  ],
  'arms-up': [
    '           ▄▄           ',
    '  ╲  ▗▄▄▄▄▄██▄▄▄▄▄▖  ╱  ',
    '   ╲ ▐ ────────── ▌ ╱   ',
    '     ▐   ●    ●   ▌     ',
    '     ▐    ╰──╯    ▌     ',
    '     ▝▀▀▀▀▀▀▀▀▀▀▀▀▘     ',
    '       ▝▘      ▝▘       ',
  ],
}

export const SMALL_POSES: Record<MascotPose, SmallSegments> = {
  default: {
    r0: '  ▗▄█▄▖  ',
    r1L: ' ▐',
    r1E: ' ● ● ',
    r1R: '▌ ',
    r2L: ' ▐',
    r2M: ' ╰─╯ ',
    r2R: '▌ ',
    r3: '  ▝▀▀▀▘  ',
  },
  'look-left': {
    r0: '  ▗▄█▄▖  ',
    r1L: ' ▐',
    r1E: '●  ● ',
    r1R: '▌ ',
    r2L: ' ▐',
    r2M: ' ╰─╯ ',
    r2R: '▌ ',
    r3: '  ▝▀▀▀▘  ',
  },
  'look-right': {
    r0: '  ▗▄█▄▖  ',
    r1L: ' ▐',
    r1E: ' ●  ●',
    r1R: '▌ ',
    r2L: ' ▐',
    r2M: ' ╰─╯ ',
    r2R: '▌ ',
    r3: '  ▝▀▀▀▘  ',
  },
  'arms-up': {
    r0: ' ╲▗▄█▄▖╱ ',
    r1L: '╲▐',
    r1E: ' ● ● ',
    r1R: '▌╱',
    r2L: ' ▐',
    r2M: ' ╰─╯ ',
    r2R: '▌ ',
    r3: '  ▝▀▀▀▘  ',
  },
}

export const SMALL_LEGS = '  ▝▘ ▝▘  '

export const APPLE_EYES: Record<MascotPose, string> = {
  default: ' ● ● ',
  'look-left': '●  ● ',
  'look-right': ' ●  ●',
  'arms-up': ' ● ● ',
}

