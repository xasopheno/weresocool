import { mainReducer } from '../../app/reducers/reducer';
import { testStore } from '../../app/store';
import { Dispatch, ResponseType } from '../../app/actions/actions';
import { language_template } from '../../app/components/Editor/language_template';
import { makeMarker } from '../../app/components/Editor/markers';
import { useReducer } from 'react';
import axios from 'axios';
import MockAdapter from 'axios-mock-adapter';
import { renderHook, act } from '@testing-library/react-hooks';

describe('Reducer Tests', () => {
  it('iterate through editor types', () => {
    const reducer = mainReducer;
    let store = testStore;
    expect(store.editor).toEqual(0);
    store = reducer(store, { _k: 'Increment_Editor_Type', editor: 1 });
    expect(store.editor).toEqual(1);
    store = reducer(store, { _k: 'Increment_Editor_Type', editor: 2 });
    expect(store.editor).toEqual(2);
    store = reducer(store, { _k: 'Increment_Editor_Type', editor: 0 });
    expect(store.editor).toEqual(0);
  });
  it('Set_Error_Message', () => {
    const reducer = mainReducer;
    let store = testStore;
    expect(store.errorMessage).toEqual('');
    store = reducer(store, { _k: 'Set_Error_Message', message: 'hello' });
    expect(store.errorMessage).toEqual('hello');
    store = reducer(store, { _k: 'Reset_Error_Message' });
    expect(store.errorMessage).toEqual('');
  });

  it('Set_Markers, Unset_Markers', () => {
    const reducer = mainReducer;
    let store = testStore;
    expect(store.markers).toEqual([]);
    const markers = [makeMarker(0, 0, 5)];
    store = reducer(store, {
      _k: 'Set_Markers',
      line: 0,
      column: 0,
      n_lines: 5,
    });
    expect(store.markers).toEqual(markers);
    store = reducer(store, { _k: 'Reset_Markers' });
    expect(store.markers).toEqual([]);
  });

  it('Set_Language', () => {
    const reducer = mainReducer;
    let store = testStore;
    expect(store.language).toEqual(language_template);
    const language = 'test test test';
    store = reducer(store, { _k: 'Set_Language', language });
    expect(store.language).toEqual(language);
    store = reducer(store, { _k: 'Reset_Language' });
    expect(store.language).toEqual(language_template);
    expect(store.markers).toEqual([]);
  });
});

describe('Fetch Tests', () => {
  it('Render: Network Error', async () => {
    const mock = new MockAdapter(axios);
    mock.onPost().networkError();

    const { result } = renderHook(() => useReducer(mainReducer, testStore));
    const [store, rawDispatch] = result.current;
    const dispatch = new Dispatch(rawDispatch);
    await act(async () => {
      await dispatch.onRender(store.language);
    });
    expect(result.current[0].backend.state).toEqual('bad');
  });

  it('Render: RenderSuccess', async () => {
    const mock = new MockAdapter(axios);
    const response = { RenderSuccess: 'Success' };
    mock.onPost().reply(200, response);

    const { result } = renderHook(() => useReducer(mainReducer, testStore));
    const [store, rawDispatch] = result.current;
    const dispatch = new Dispatch(rawDispatch);
    await act(async () => {
      await dispatch.onRender(store.language);
    });

    const state = result.current[0];
    expect(state.backend.state).toEqual('good');
    expect(state.render).toEqual(ResponseType.RenderSuccess);
  });

  it('Render: ParseError', async () => {
    const mock = new MockAdapter(axios);
    const response = {
      ParseError: { message: 'Unexpected Token', line: 14, column: 15 },
    };
    mock.onPost().reply(200, response);

    const { result } = renderHook(() => useReducer(mainReducer, testStore));
    const [store, rawDispatch] = result.current;
    const dispatch = new Dispatch(rawDispatch);
    await act(async () => {
      await dispatch.onRender(store.language);
    });

    const state = result.current[0];
    const expected_markers = [makeMarker(14, 15, 20)];
    expect(state.backend.state).toEqual('good');
    expect(state.render).toEqual(ResponseType.ParseError);
    expect(state.markers).toEqual(expected_markers);
    expect(state.errorMessage).toEqual('Line: 14 | Column 15');
  });

  it('Render: IdError', async () => {
    const mock = new MockAdapter(axios);
    const response = {
      IdError: { id: 'thing' },
    };
    mock.onPost().reply(200, response);

    const { result } = renderHook(() => useReducer(mainReducer, testStore));
    const [store, rawDispatch] = result.current;
    const dispatch = new Dispatch(rawDispatch);
    await act(async () => {
      await dispatch.onRender(store.language);
    });

    const state = result.current[0];
    expect(state.backend.state).toEqual('good');
    expect(state.render).toEqual(ResponseType.IdError);
    expect(state.markers).toEqual([]);
    expect(state.errorMessage).toEqual('thing');
  });

  it('Render: IndexError', async () => {
    const mock = new MockAdapter(axios);
    const response = {
      IndexError: {
        len_list: 7,
        index: 8,
        message: 'index 8 is greater than length of list 7',
      },
    };
    mock.onPost().reply(200, response);

    const { result } = renderHook(() => useReducer(mainReducer, testStore));
    const [store, rawDispatch] = result.current;
    const dispatch = new Dispatch(rawDispatch);
    await act(async () => {
      await dispatch.onRender(store.language);
    });

    const state = result.current[0];
    expect(state.backend.state).toEqual('good');
    expect(state.render).toEqual(ResponseType.IndexError);
    expect(state.markers).toEqual([]);
    expect(state.errorMessage).toEqual(
      'index 8 is greater than length of list 7'
    );
  });
  it('Render: MsgError', async () => {
    const mock = new MockAdapter(axios);
    const response = {
      Msg: 'I am a message',
    };
    mock.onPost().reply(200, response);

    const { result } = renderHook(() => useReducer(mainReducer, testStore));
    const [store, rawDispatch] = result.current;
    const dispatch = new Dispatch(rawDispatch);
    await act(async () => {
      await dispatch.onRender(store.language);
    });

    const state = result.current[0];
    expect(state.backend.state).toEqual('good');
    expect(state.render).toEqual(ResponseType.MsgError);
    expect(state.markers).toEqual([]);
    console.log(state.errorMessage);
    expect(state.errorMessage).toEqual('I am a message');
  });
});
