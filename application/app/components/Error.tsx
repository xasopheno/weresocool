import React from 'react';
import styled from 'styled-components';
import { ResponseType } from '../actions/actions';

export const ErrorDescription = (props: {
  errorMessage: string;
  responseState: ResponseType;
}): React.ReactElement => {
  switch (props.responseState) {
    case ResponseType.RenderSuccess:
      return <div />;
    case ResponseType.ParseError:
      return (
        <ErrorType id={'errorDescription'}>
          UnexpectedToken: <ErrorMessage>{props.errorMessage}</ErrorMessage>
        </ErrorType>
      );
    case ResponseType.IdError:
      return (
        <ErrorType id={'errorDescription'}>
          Name Not Found: <ErrorMessage>{props.errorMessage}</ErrorMessage>
        </ErrorType>
      );
    case ResponseType.IndexError:
      return (
        <ErrorType>
          IndexError:{' '}
          <ErrorMessage id={'errorDescription'}>
            {props.errorMessage}
          </ErrorMessage>
        </ErrorType>
      );
    case ResponseType.MsgError:
      return (
        <ErrorType id={'errorDescription'}>
          Error: <ErrorMessage>{props.errorMessage}</ErrorMessage>
        </ErrorType>
      );
    default:
      return (
        <ErrorType id={'errorDescription'}>
          Error: <ErrorMessage>Error</ErrorMessage>
        </ErrorType>
      );
  }
};

const ErrorType = styled.p`
  width: 80%;
  margin-left: 10%;
  font-family: 'Courier New', Courier, monospace;
  font-weight: bold;
  text-align: center;
  background-color: #454343;
  color: ${(props) => props.color};
  font-size: 1.7em;
  margin-top: 0;
  margin-bottom: 0;
  padding-top: 0;
  padding-bottom: 0;
  color: mistyrose;
`;

const ErrorMessage = styled.span`
  font-family: 'Courier New', Courier, monospace;
  font-weight: bold;
  background-color: #111101;
  color: ${(props) => props.color};
  margin-top: 0;
  margin-bottom: 0;
  padding-top: 0;
  padding-bottom: 0;
  padding-left: 1em;
  padding-right: 1em;
  color: cornsilk;
`;
