import React, { useContext } from 'react';
import { TopBox, ButtonBox, Button, RightButton, VimBox } from './style';
import { DispatchContext } from '../actions/actions';
import { GlobalContext, Editors } from '../store';
import ReactTooltip from 'react-tooltip';
import { Render } from './Render';
import styled from 'styled-components';
import { useCurrentWidth } from '../utils/width';

const SliderContainer = styled.div`
  padding-top: 5px;
  width: 100%;
`;

const Slider = styled.input`
  -webkit-appearance: none;
  width: 150px;
  background: transparent;
  opacity: 0.7;
  -webkit-transition: 0.1s;
  transition: opacity 0.1s;

  :focus {
    outline: none;
    opacity: 1;
  }

  ::-webkit-slider-runnable-track {
    height: 0.9rem;
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
    height: 1.5rem;
    width: 0.5rem;
    background: #edd;
    border: 1px solid;
    margin-top: -5px;
    border-radius: 3px;
    border-color: #eed;
    cursor: pointer;
  }

  // // :focus::-webkit-slider-thumb {
  // // box-shadow: 0px 0px 1px 1px #fff;
  // // }

  // -webkit-appearance: none;
  // width: 150px;
  // height: 10px;
  // background: goldenrod;
  // outline: none;
  // opacity: 0.7;
  // -webkit-transition: 0.2s;
  // transition: opacity 0.2s;

  // :hover {
  // opacity: 1;
  // }

  // ::-webkit-slider-thumb {
  // -webkit-appearance: none; /* Override default look */
  // appearance: none;
  // width: 20px; /* Set a specific slider handle width */
  // height: 20px; /* Slider handle height */
  // background: #cbb; /* Green background */
  // cursor: pointer; /* Cursor on hover */
  // }
`;

const stub = () => {};

type Props = { handleLoad: () => void };

export const Controls = (props: Props): React.ReactElement => {
  const store = useContext(GlobalContext);
  const dispatch = useContext(DispatchContext);
  const width = useCurrentWidth();

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
            await dispatch.onRender(store.language, store.volume);
            dispatch.setEditorFocus(store.editor_ref);
          }}
          disabled={store.printing}
        >
          {width > 1000 ? 'Play' : 'P'}
        </Button>
        <Button
          data-tip="⌘+Enter"
          id={'stopButton'}
          onClick={dispatch.onStop}
          disabled={store.printing}
        >
          {width > 1000 ? 'Stop' : 'S'}
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
          {width > 1000 ? 'Load' : 'L'}
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
          {width > 1000 ? 'Save' : 'S'}
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
            onChange={(e) => {
              dispatch.onVolumeChange(parseFloat(e.target.value));
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
