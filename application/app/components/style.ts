import styled from 'styled-components';

export const StateText = styled.p`
  font-family: 'Courier New', Courier, monospace;
  color: ${(props: { color: string }): string => props.color};
  font-size: 1em;
  margin-top: 0;
  margin-bottom: 0;
  padding-top: 0;
  padding-bottom: 0;
  color: gold;
`;

export const Space = styled.div`
  background-color: #454343;
  height: 100vh;
`;

export const VimBox = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: flex-end;
  margin-right: 10%;
  margin-bottom: 10px;
`;

export const VimText = styled.label`
  font-family: 'Courier New', Courier, monospace;
  text-align: bottom;
  color: #cbb;
  font-size: 1em;
  padding-left: 0.2em;
`;

export const CheckBox = styled.input`
  vertical-align: middle;
  margin-bottom: 0px;
`;

export const ButtonBox = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: flex-start;
  margin-left: 10%;
  margin-bottom: 10px;
`;

export const Button = styled.button`
  font-size: 1.3em;
  background-color: #454343;
  margin-right: 1em;
  color: #edd;
  &:visited {
    outline: none;
  }
  &:active {
    outline: none;
    background-color: #555373;
  }
  &:focus {
    outline: none;
  }
  &:hover {
    cursor: pointer;
    background-color: #555373;
  }
`;

export const RightButton = styled.button`
  font-size: 1.3em;
  background-color: #454343;
  margin-left: 1em;
  color: #edd;
  &:visited {
    outline: none;
  }
  &:active {
    outline: none;
    background-color: #555373;
  }
  &:focus {
    outline: none;
  }
  &:hover {
    cursor: pointer;
    background-color: #555373;
  }
`;

export const TopBox = styled.div`
  display: flex;
  justify-content: space-between;
`;
