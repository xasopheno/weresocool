import { IMarker } from 'react-ace';

export const makeMarker = (
  line: number,
  column: number,
  n_lines: number
): IMarker => {
  line -= 1;
  return {
    startRow: line,
    startCol: column,
    endRow: n_lines,
    endCol: 0,
    type: 'text',
    className: 'error',
  };
};
