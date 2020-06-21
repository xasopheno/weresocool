import React, { useContext } from 'react';
import styled from 'styled-components';
import { DemoList } from './tutorial_list';
import { DispatchContext } from '../actions/actions';
import { GlobalContext } from '../store';

const Modal = styled.div`
  position: absolute;
  background-color: #454343;
  opacity: 0.95;
  top: 0;
  bottom: 0;
  right: 0;
  left: 0;
  z-index: 10;
`;

const Title = styled.h1`
  font-size: 40px;
  margin-top: 60px;
  text-align: center;
  color: #edd;
`;
const Section = styled.p`
  font-size: 20px;
  text-align: center;
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

const TitleContainer = styled.div`
  overflow-y: scroll;
  height: 700px;
`;

export interface DemoData {
  show: boolean;
  setShow: (b: boolean) => void;
  data: DemoList;
  title: string;
  folder: string;
}

export const Demo = (props: { demoData: DemoData }): React.ReactElement => {
  const store = useContext(GlobalContext);
  const dispatch = useContext(DispatchContext);
  const chooseTutorial = async (filename: string) => {
    props.demoData.setShow(false);
    await dispatch.onDemo(filename, props.demoData.folder);
    dispatch.setEditorFocus(store.editor_ref);
  };

  const makeDemos = (): React.ReactElement => {
    return (
      <TitleContainer>
        {props.demoData.data.map((tutorial, i) => {
          return (
            <Section key={i} onClick={() => chooseTutorial(tutorial.filename)}>
              {i}. {tutorial.text}
            </Section>
          );
        })}
      </TitleContainer>
    );
  };
  if (props.demoData.show) {
    return (
      <Modal>
        <Title>{props.demoData.title}</Title>
        {makeDemos()}
        <Button onClick={() => props.demoData.setShow(false)}>X</Button>
      </Modal>
    );
  } else {
    return <div />;
  }
};
