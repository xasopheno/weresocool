import React, { useContext } from 'react';
import { Button } from './style';
import { DispatchContext } from '../actions/actions';
import { GlobalContext } from '../store';
import styled from 'styled-components';
import { useCurrentWidth } from '../utils/width';
import ReactTooltip from 'react-tooltip';

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

const Section = styled.p`
  font-size: 50px;
  // text-align: center;
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

const TextContainer = styled.div`
  height: 100%;
  width: 100%;
  position: absolute;
  top: 30%;
  left: 50%;
`;

export const Render = (): React.ReactElement => {
  const width = useCurrentWidth();
  const store = useContext(GlobalContext);
  const [showRenderModal, setShowRenderModal] = React.useState(false);

  const options: RenderModalOptions = {
    setShow: setShowRenderModal,
    show: showRenderModal,
  };

  return (
    <div>
      <RenderModal options={options} />
      <Button
        data-tip="Render"
        id={'printButton'}
        onClick={() => {
          setShowRenderModal(true);
        }}
        disabled={store.printing}
      >
        {width > 800 ? 'Render' : 'R'}
      </Button>
    </div>
  );
};

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
      <Modal id={'renderModal'}>
        <TextContainer>
          <Section
            id={'mp3Button'}
            onClick={async () => {
              await dispatch.onStop();
              props.options.setShow(false);
              await dispatch.onPrint(store.language, 'mp3');
              dispatch.setEditorFocus(store.editor_ref);
            }}
          >
            Mp3
          </Section>
          <Section
            id={'wavButton'}
            onClick={async () => {
              await dispatch.onStop();
              props.options.setShow(false);
              await dispatch.onPrint(store.language, 'wav');
              dispatch.setEditorFocus(store.editor_ref);
            }}
          >
            Wav
          </Section>
        </TextContainer>
        <CloseModalButton onClick={() => props.options.setShow(false)}>
          X
        </CloseModalButton>
      </Modal>
    );
  } else {
    return <div />;
  }
};
