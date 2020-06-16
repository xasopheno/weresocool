import React, { useContext } from 'react';
import styled from 'styled-components';
import { DemoList } from './tutorial_list';
import { DispatchContext } from '../actions/actions';

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
  margin-top: 120px;
  text-align: center;
  color: #edd;
`;
const Section = styled.p`
  font-size: 30px;
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

export interface DemoData {
  show: boolean;
  setShow: (b: boolean) => void;
  data: DemoList;
  title: string;
  folder: string;
}

export const Demo = (props: { demoData: DemoData }): React.ReactElement => {
  const dispatch = useContext(DispatchContext);
  const chooseTutorial = async (filename: string) => {
    props.demoData.setShow(false);
    await dispatch.onDemo(filename, props.demoData.folder);
  };

  const makeDemos = (): React.ReactElement => {
    return (
      <div>
        {props.demoData.data.map((tutorial, i) => {
          return (
            <Section key={i} onClick={() => chooseTutorial(tutorial.filename)}>
              {tutorial.text}
            </Section>
          );
        })}
      </div>
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
