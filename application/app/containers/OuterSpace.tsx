import React, { useContext, useRef } from 'react';
import { Logo } from '../components/Title';
import { Ratios } from '../components/Ratios';
import { Controls } from '../components/Controls';
import { Editor } from '../components/Editor/editor';
import { LED } from '../components/Backend';
import { ErrorDescription } from '../components/Error';
import { useCurrentWidth } from '../utils/width';
import { GlobalContext } from '../store';
import { DispatchContext } from '../actions/actions';
import { remote } from 'electron';
import styled from 'styled-components';

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

  const Version = styled.p`
    position: absolute;
    right: 0;
    margin-right: 5px;
    top: 2;
    color: #111111;
    font-family: monospace;
  `;

  return (
    <GlobalContext.Provider value={store}>
      <input
        ref={fileInput}
        id={'fileLoadInput'}
        type="file"
        accept=".socool"
        style={{ display: 'none', visibility: 'hidden' }}
        onChange={(e) => dispatch.onFileLoad(e)}
      />
          <Version>
            {`v${remote.app.getVersion()}`}
          </Version>

      <LED state={store.backend.state} />
      <Logo />;
      <Ratios width={width} />
      <Controls handleLoad={handleLoad} />
      <Editor handleLoad={handleLoad} />
      <ErrorDescription
        errorMessage={store.errorMessage}
        responseState={store.render}
      />
    </GlobalContext.Provider>
  );
};
