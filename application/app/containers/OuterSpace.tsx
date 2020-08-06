import React, { useState, useContext, useRef } from 'react';
import { Logo } from '../components/Title';
import { Ratios } from '../components/Ratios';
import { ButtonBar } from '../components/ButtonBar';
import { Controls } from '../components/Controls';
import { Editor } from '../components/Editor/editor';
import { LED } from '../components/Backend';
import { ErrorDescription } from '../components/Error';
import { useCurrentWidth } from '../utils/width';
import { GlobalContext } from '../store';
import { DispatchContext } from '../actions/actions';
import { remote } from 'electron';
import styled from 'styled-components';
// @ts-ignore
import { Bars } from 'svg-loaders-react';

const Version = styled.p`
  position: absolute;
  right: 0;
  margin-right: 5px;
  top: 2;
  color: #111111;
  font-family: monospace;
`;

const SpinnerContainer = styled.div`
  height: 100%;
  width: 100%;
  position: absolute;
  top: 40%;
  left: 45%;
  z-index: 100;
`;

const SpinnerText = styled.p`
  font-family: 'Courier New', Courier, monospace;
  font-size: 2em;
  color: goldenrod;
`;

interface SpinnerProps {
  show: boolean;
}

const Spinner = (props: SpinnerProps): React.ReactElement => {
  if (props.show) {
    return (
      <SpinnerContainer id={'spinner'}>
        <SpinnerText>Rendering.....</SpinnerText>
        <Bars stroke={'goldenrod'} heigth={200} width={200} />
      </SpinnerContainer>
    );
  } else {
    return <></>;
  }
};

export const OuterSpace = (): React.ReactElement => {
  const width = useCurrentWidth();
  const store = useContext(GlobalContext);
  const dispatch = useContext(DispatchContext);
  const fileInput = useRef<HTMLInputElement>(null);

  const handleLoad = () => {
    if (fileInput && fileInput.current) {
      fileInput.current.click();
    }
  };

  return (
    <GlobalContext.Provider value={store}>
      <input
        ref={fileInput}
        id={'fileLoadInput'}
        type="file"
        accept=".socool"
        style={{ display: 'none', visibility: 'hidden' }}
        onChange={(e) => {
          dispatch.onFileLoad(e);
          dispatch.setEditorFocus(store.editor_ref);
        }}
      />
      <Spinner show={store.printing} />
      <Version>{`v${remote.app.getVersion()}`}</Version>
      <LED state={store.backend.state} />
      <Logo />
      <Ratios width={width} />
      <ButtonBar width={width} />
      <Controls handleLoad={handleLoad} />
      <Editor handleLoad={handleLoad} />

      <ErrorDescription
        errorMessage={store.errorMessage}
        responseState={store.render}
      />
    </GlobalContext.Provider>
  );
};
