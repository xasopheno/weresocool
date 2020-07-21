import React from 'react';
import styled from 'styled-components';
// import path from 'path';
// import { remote } from 'electron';
// import { Demo, DemoData } from './Tutorial';
// import { tutorial_list, album_list } from './tutorial_list';
import { RatioChart } from './RatioChart';

const RSpace = styled.div`
  position: absolute;
  top: 10;
  right: 0;
  display: flex;
  flex-direction: column;
  font-family: 'Courier New', Courier, monospace;
  font-size: 1.1em;
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
    return <div />;
  } else {
    return <div />;
  }
};
