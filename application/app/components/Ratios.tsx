import React from 'react';
import styled from 'styled-components';
import { RatioChart } from './RatioChart';

const RSpace = styled.div`
  position: absolute;
  right: 0;
  top: 70px;
  display: flex;
  flex-direction: column;
  font-family: 'Courier New', Courier, monospace;
  margin-right: 1em;
  text-align: center;
  border: 5px ridge goldenrod;
`;

export const Ratios = (props: { width: number }): React.ReactElement | null => {
  if (props.width > 1000) {
    return (
      <div>
        <RSpace id="ratios">
          <RatioChart />
        </RSpace>
      </div>
    );
  } else if (props.width > 700) {
    return null;
  } else {
    return null;
  }
};
