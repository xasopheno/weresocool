import { Store } from '../store';
import { Action } from '../actions/actions';
import { makeMarker } from '../components/Editor/markers';
import { language_template } from '../components/Editor/language_template';

// eslint-disable-next-line
function exhaustive(_param: never) {}

export const mainReducer: React.Reducer<Store, Action> = (
  state,
  action
): Store => {
  switch (action._k) {
    case 'Increment_Editor_Type':
      return { ...state, editor: action.editor };
    case 'Increment_Demo_Index':
      return { ...state, demoIdx: (state.demoIdx + 1) % action.len };
    case 'Set_Error_Message':
      return { ...state, errorMessage: action.message };
    case 'Reset_Error_Message':
      return { ...state, errorMessage: '' };
    case 'Set_Markers':
      return {
        ...state,
        markers: [makeMarker(action.line, action.column, action.n_lines)],
      };
    case 'Reset_Markers':
      return {
        ...state,
        markers: [],
      };
    case 'Set_Language':
      return {
        ...state,
        language: action.language,
      };

    case 'Reset_Language':
      return {
        ...state,
        language: language_template,
      };
    case 'Set_Render_State':
      return {
        ...state,
        render: action.state,
      };
    case 'Backend':
      switch (action.fetch.state) {
        case 'loading':
          return { ...state, backend: { state: 'loading' } };
        case 'good':
          return { ...state, backend: { state: 'good' } };
        case 'bad':
          return {
            ...state,
            backend: { state: 'bad', error: Error('bad state') },
          };
        default: {
          exhaustive(action.fetch);
        }
      }
      break;
    default: {
      exhaustive(action);
    }
  }

  throw new Error('Impossible');
};
