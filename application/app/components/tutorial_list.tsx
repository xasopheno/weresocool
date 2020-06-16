export type DemoList = Array<{ filename: string; text: string }>;

export const tutorial_list: DemoList = [
  { filename: '01_major_scale.socool', text: '1. Major Scale' },
  {
    filename: '02_moving_things_around.socool',
    text: '2. Pipe',
  },
  { filename: '03_sequences.socool', text: '3. Pipe and Sequences' },
  { filename: '04_overlay.socool', text: '4. Overlay [op1, op2...]' },
  { filename: '05_O.socool', text: '5. O[(Fm, Fa, Gm, Pa), ...]' },
  { filename: '06_fit_length.socool', text: '6. Op2 > FitLength Op1' },
];

export const album_list: DemoList = [
  { filename: 'day_5.socool', text: '1. Day 5' },
  { filename: 'table.socool', text: '2. Table' },
  { filename: 'day_3.socool', text: '3. Goodbye, Glacier Bay' },
  // { filename: 'herring.socool', text: '4. Herring' },
  // { filename: 'how_to_move.socool', text: '4. How To Move' },
  // { filename: 'modern_modem.socool', text: '4. Modern Modem' },
  // { filename: 'arcs.socool', text: '5. Arcs' },
  // { filename: 'antonis.socool', text: '6. Antonis' },
  // { filename: 'delayed.socool', text: '7. Delayed' },
  // { filename: 'madness.socool', text: '8. Madness' },
  // { filename: 'Marichan.socool', text: '1. Marichan' },
];
