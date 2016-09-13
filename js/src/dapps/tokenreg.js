import ReactDOM from 'react-dom';
import React from 'react';
import { Provider } from 'react-redux';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import store from './tokenreg/store';
import Container from './tokenreg/Container';

import './style.css';
import './tokenreg.html';

ReactDOM.render(
  (
    <Provider store={ store }>
      <Container />
    </Provider>
  ),
  document.querySelector('#container')
);
