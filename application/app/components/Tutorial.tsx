import React from 'react';
import styled from 'styled-components';

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
`;
const Section = styled.p`
  font-size: 30px;
  text-align: center;
`;
const Button = styled.div`
  position: absolute;
  right: 0;
  bottom: 0;
  margin: 80px;
  font-size: 80px;
`;

type Props = { show: boolean; setShow: (b: boolean) => void };

export const Tutorial = (props: Props): React.ReactElement => {
  if (props.show) {
    return (
      <Modal>
        <Title>Tutorials</Title>
        <Section>1. Making Sound</Section>
        <Section>2. Moving Things Around</Section>
        <Section>3. Sequences</Section>
        <Section>4. Overlay</Section>
        <Section>5. O[]</Section>
        <Section>6. FitLength</Section>
        <Button onClick={() => props.setShow(false)}>X</Button>
      </Modal>
    );
  } else {
    return <div />;
  }
};

