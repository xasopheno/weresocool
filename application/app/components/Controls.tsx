import React, { useContext } from 'react';
import { TopBox, ButtonBox, Button, RightButton, VimBox } from './style';
import { DispatchContext } from '../actions/actions';
import { GlobalContext, Editors } from '../store';

const stub = () => {};

type Props = { handleLoad: () => void };

export const Controls = (props: Props): React.ReactElement => {
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

      <ButtonBox>
        <Button
          id={'renderButton'}
          onClick={() => dispatch.onRender(store.language)}
        >
          Render
        </Button>
        <Button id={'stopButton'} onClick={dispatch.onStop}>
          Stop
        </Button>
        <Button id={'loadButton'} onClick={props.handleLoad}>
          Load
        </Button>
        <Button
          id={'saveButton'}
          onClick={() => dispatch.onFileSave(store.language)}
        >
          Save
        </Button>
      </ButtonBox>

      <VimBox>
        <RightButton id={'resetButton'} onClick={dispatch.onResetLanguage}>
          Reset
        </RightButton>
        <RightButton
          id={'editorButton'}
          onClick={() => dispatch.onIncrementEditorType()}
        >
          {Editors[store.editor].name}
        </RightButton>
      </VimBox>
    </TopBox>
  );
};
