import React, { useContext } from 'react';
import { TopBox, ButtonBox, Button, RightButton, VimBox } from './style';
import { DispatchContext } from '../actions/actions';
import { GlobalContext, Editors } from '../store';
import ReactTooltip from 'react-tooltip';
import { Render } from './Render';
import { useCurrentWidth } from '../utils/width';

const stub = () => {};

type Props = { handleLoad: () => void };

export const Controls = (props: Props): React.ReactElement => {
  const width = useCurrentWidth();
  const store = useContext(GlobalContext);
  const dispatch = useContext(DispatchContext);

  return (
    <TopBox>
      <input
        type="file"
        accept=".socool"
        style={{ display: 'none', visibility: 'hidden' }}
        onChange={stub}
      />
      <ReactTooltip />

      <ButtonBox>
        <Button
          data-tip="Shift+Enter"
          id={'playButton'}
          onClick={async () => {
            await dispatch.onRender(store.language);
            dispatch.setEditorFocus(store.editor_ref);
          }}
          disabled={store.printing}
        >
          {width > 650 ? 'Play' : 'P'}
        </Button>
        <Button
          data-tip="⌘+Enter"
          id={'stopButton'}
          onClick={dispatch.onStop}
          disabled={store.printing}
        >
          {width > 650 ? 'Stop' : 'S'}
        </Button>

        <Render />

        <Button
          data-tip="⌘+L"
          id={'loadButton'}
          onClick={() => {
            props.handleLoad();
          }}
          disabled={store.printing}
        >
          {width > 650 ? 'Load' : 'L'}
        </Button>

        <Button
          data-tip="⌘+S"
          id={'saveButton'}
          onClick={() => {
            dispatch.onFileSave(store.language);
            dispatch.setEditorFocus(store.editor_ref);
          }}
          disabled={store.printing}
        >
          {width > 650 ? 'Save' : 'S'}
        </Button>
      </ButtonBox>

      <VimBox>
        <RightButton
          id={'resetButton'}
          onClick={() => {
            dispatch.onResetLanguage();
            dispatch.setEditorFocus(store.editor_ref);
          }}
          disabled={store.printing}
        >
          Reset
        </RightButton>
        <RightButton
          id={'editorButton'}
          onClick={() => {
            dispatch.onIncrementEditorType(store.editor);
            dispatch.setEditorFocus(store.editor_ref);
          }}
          disabled={store.printing}
        >
          {Editors[store.editor].name}
        </RightButton>
      </VimBox>
    </TopBox>
  );
};
