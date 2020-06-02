import React from 'react';
import { BrowserRouter } from 'react-router-dom';
import { hot } from 'react-hot-loader/root';
import Routes from '../Routes';
import { Store } from '../store';

type Props = { initialStore: Store };

const Root = (props: Props) => (
  <BrowserRouter>
    <Routes initialStore={props.initialStore} />
  </BrowserRouter>
);

export default hot(Root);
