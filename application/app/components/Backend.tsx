import React from 'react';
import styled from 'styled-components';

const LEDBulbGood = styled.div`
  background-color: springgreen;
  position: absolute;
  top: 0;
  right: 0;
  width: 10px;
  height: 10px;
`;

const LEDBulbBad = styled.div`
  background-color: indianred;
  position: absolute;
  top: 0;
  right: 0;
  width: 20px;
  height: 20px;
`;

type Props = { state: 'good' | 'bad' | 'loading' };

export const LED = (props: Props): React.ReactElement => {
  if (props.state === 'bad') {
    return <LEDBulbBad id={'led_bad'} />;
  } else {
    return <LEDBulbGood id={'led_good'} />;
  }
};
