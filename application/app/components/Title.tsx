import React from 'react';
import styled from 'styled-components';

export const Logo = (props: { width: number }): React.ReactElement | null => {
  return (
    <div id={'outerSpace'}>
      <Title>WereSoCool</Title>

      {props.width > 550 ? (
        <SubTitle>Make cool sounds. Impress your friends/pets/plants.</SubTitle>
      ) : (
        <SubTitle>Make cool sounds.</SubTitle>
      )}
    </div>
  );
};

export const Title = styled.h1`
  font-family: 'Courier New', Courier, monospace;
  text-align: center;
  padding-top: 10px;
  color: #edd;
  font-size: 1.5em;
`;

export const SubTitle = styled.p`
  font-family: 'Courier New', Courier, monospace;
  text-align: center;
  color: #cbb;
  font-size: 1em;
`;
