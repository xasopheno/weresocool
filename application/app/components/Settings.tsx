import React, { useContext } from 'react';
import styled from 'styled-components';
import { DispatchContext } from '../actions/actions';
import { GlobalContext, Editors } from '../store';
import { remote } from 'electron';

const Modal = styled.div`
  position: absolute;
  background-color: #454343;
  opacity: 0.95;
  top: 0;
  bottom: 0;
  right: 0;
  left: 0;
  z-index: 200;
`;

const Title = styled.h1`
  font-size: 40px;
  margin-top: 60px;
  text-align: center;
  color: #edd;
`;
const Section = styled.p`
  font-size: 20px;
  text-align: left;
  color: #edd;
  :hover {
    text-decoration: underline;
  }
`;

const Button = styled.div`
  position: absolute;
  right: 0;
  bottom: 0;
  margin: 80px;
  font-size: 80px;
  color: #edd;
`;
const SectionContainer = styled.div`
  display: flex;
  flex-direction: column;
  margin-left: 10vw;
`;

export interface SettingsData {
  show: boolean;
  setShow: (b: boolean) => void;
}

export const Settings = (props: {
  settingsData: SettingsData;
}): React.ReactElement => {
  const store = useContext(GlobalContext);
  const dispatch = useContext(DispatchContext);
  const openDialog = async () => {
    const path = await remote.dialog.showOpenDialog({
      properties: ['openDirectory'],
    });
    console.log(path);
    if (path) {
      dispatch.onSetWorkingPath(path.filePaths[0]);
    }
  };

  if (props.settingsData.show) {
    return (
      <Modal id={'settingsModal'}>
        <Title>Settings</Title>
        <SectionContainer>
          <Section
            id={'editorButton'}
            onClick={() => dispatch.onIncrementEditorType(store.editor)}
          >
            Editor: {Editors[store.editor].name}
          </Section>
          <Section
            id={'workingPathButton'}
            onClick={async () => await openDialog()}
          >
            Working Path: &quot;{store.working_path}&quot;
          </Section>
        </SectionContainer>
        <Button onClick={() => props.settingsData.setShow(false)}>X</Button>
      </Modal>
    );
  } else {
    return <></>;
  }
};
