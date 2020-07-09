import React, { useContext, useState } from 'react';
import { Button } from './style';
import { DispatchContext } from '../actions/actions';
import { GlobalContext } from '../store';
import styled from 'styled-components';

export const Render = (): React.ReactElement => {
  const store = useContext(GlobalContext);
  const dispatch = useContext(DispatchContext);
  const [showRenderModal, setShowRenderModal] = React.useState(false);

  const options: RenderModalOptions = {
    setShow: setShowRenderModal,
    show: showRenderModal,
  };

  return (
    <div>
      <RenderModal options={options} />
      <Button
        id={'printButton'}
        onClick={() => {
          setShowRenderModal(true);
        }}
        disabled={store.printing}
      >
        Render
      </Button>
    </div>
  );
};

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
  font-size: 50px;
  text-align: center;
  color: #edd;
  :hover {
    text-decoration: underline;
  }
`;

const CloseModalButton = styled.div`
  position: absolute;
  right: 0;
  bottom: 0;
  margin: 80px;
  font-size: 80px;
  color: #edd;
`;

export interface RenderModalOptions {
  show: boolean;
  setShow: (b: boolean) => void;
}

export const RenderModal = (props: {
  options: RenderModalOptions;
}): React.ReactElement => {
  const store = useContext(GlobalContext);
  const dispatch = useContext(DispatchContext);
  if (props.options.show) {
    return (
      <Modal>
        <Title>Render Options</Title>
        <Section
          onClick={async () => {
            await dispatch.onStop();
            props.options.setShow(false);
            await dispatch.onPrint(store.language);
            dispatch.setEditorFocus(store.editor_ref);
          }}
        >
          Mp3
        </Section>
        <Section>Wav</Section>
        <CloseModalButton onClick={() => props.options.setShow(false)}>
          X
        </CloseModalButton>
      </Modal>
    );
  } else {
    return <div />;
  }
};
