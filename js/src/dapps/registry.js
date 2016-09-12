import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import store from './registry/store';
import Container from './registry/Container';

import './style.css';
import './registry.html';

ReactDOM.render(
  (
    <Provider store={ store }>
      <Container />
    </Provider>
  ),
  document.querySelector('#container')
);
