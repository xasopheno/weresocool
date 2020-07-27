import React, { useContext } from 'react';
import { TopBox, ButtonBox, Button, RightButton, VimBox } from './style';
import { DispatchContext } from '../actions/actions';
import { GlobalContext, Editors } from '../store';
import ReactTooltip from 'react-tooltip';
import { Render } from './Render';
import { useCurrentWidth } from '../utils/width';
import styled from 'styled-components';

const SliderContainer = styled.div`
  padding-top: 5px;
  width: 100%;
`;

const Slider = styled.input`
  -webkit-appearance: none;
  width: 200px;
  background: transparent;
  opacity: 0.7;
  -webkit-transition: 0.1s;
  transition: opacity 0.1s;

  :focus {
    outline: none;
    opacity: 1;
  }

  ::-webkit-slider-runnable-track {
    height: 1.3rem;
    margin: 0;
    width: 100%;
    cursor: pointer;
    background: goldenrod;
    background: linear-gradient(
      to bottom right,
      transparent 50%,
      goldenrod 50%
    );
  }

  ::-webkit-slider-thumb {
    -webkit-appearance: none;
    height: 1.7rem;
    width: 0.5rem;
    background: #edd;
    border: 1px solid;
    margin-top: -5px;
    border-radius: 3px;
    border-color: #eed;
    cursor: pointer;
  }
`;
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
        <SliderContainer>
          <Slider
            type="range"
            min="0"
            max="100"
            id="volumeSlider"
            value={store.volume}
            onChange={async (e) => {
              await dispatch.onVolumeChange(parseInt(e.target.value));
            }}
            onMouseUp={() => {
              dispatch.setEditorFocus(store.editor_ref);
            }}
          />
        </SliderContainer>
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
