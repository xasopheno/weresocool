import React, { Fragment } from 'react';
import { render } from 'react-dom';
import { AppContainer as ReactHotAppContainer } from 'react-hot-loader';
import Root from './containers/Root';
import './app.global.css';
import { intialStore } from './store';

const AppContainer = process.env.PLAIN_HMR ? Fragment : ReactHotAppContainer;
document.addEventListener('DOMContentLoaded', () =>
  render(
    <AppContainer>
      <Root initialStore={intialStore} />
    </AppContainer>,
    document.getElementById('root')
  )
);
