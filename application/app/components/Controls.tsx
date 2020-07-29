import React, { useContext } from 'react';
import { TopBox, ButtonBox, Button, RightButton, VimBox } from './style';
import { DispatchContext } from '../actions/actions';
import { GlobalContext, Editors } from '../store';
import ReactTooltip from 'react-tooltip';
import { Render } from './Render';
import { useCurrentWidth } from '../utils/width';
import styled from 'styled-components';

const SliderContainer = styled.div`
  display: flex;
  flex-direction: row;
  padding-top: 5px;
  width: 100%;
`;

const VolumeText = styled.p<SliderProps>`
  margin: 0;
  padding-left: 10px;
  min-width: 30px;
  padding-top: 2px;
  color: ${(p: SliderProps) => (p.active ? 'goldenrod' : '#aaa')};
  opacity: 0.8;
`;

interface SliderProps {
  active: boolean;
}

const Slider = styled.input<SliderProps>`
  -webkit-appearance: none;
  width: 100%;
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
    background: linear-gradient(
      to bottom right,
      transparent 50%,
      ${(p: SliderProps) => (p.active ? 'goldenrod' : '#aaa')} 50%
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
  const break_point = 800;

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
          {width > break_point ? 'Play' : 'P'}
        </Button>
        <Button
          data-tip="⌘+Enter"
          id={'stopButton'}
          onClick={dispatch.onStop}
          disabled={store.printing}
        >
          {width > break_point ? 'Stop' : 'S'}
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
          {width > break_point ? 'Load' : 'L'}
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
          {width > break_point ? 'Save' : 'S'}
        </Button>
      </ButtonBox>

      <VimBox>
        <SliderContainer>
          <Slider
            active={store.volume > 0.0}
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
          <VolumeText id={'volumeText'} active={store.volume > 0.0}>
            {store.volume}
          </VolumeText>
        </SliderContainer>
        <RightButton
          data-tip="Reset to Template"
          id={'resetButton'}
          onClick={() => {
            dispatch.onResetLanguage();
            dispatch.setEditorFocus(store.editor_ref);
          }}
          disabled={store.printing}
        >
          {width > break_point ? 'Reset' : 'R'}
        </RightButton>
        <RightButton
          data-tip="Choose Editor"
          id={'editorButton'}
          onClick={() => {
            dispatch.onIncrementEditorType(store.editor);
            dispatch.setEditorFocus(store.editor_ref);
          }}
          disabled={store.printing}
        >
          {width > break_point
            ? Editors[store.editor].name
            : Editors[store.editor].name.charAt(0)}
        </RightButton>
      </VimBox>
    </TopBox>
  );
};
