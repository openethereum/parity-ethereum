import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';

import store from './store';
import Container from './Container';

import '../style.css';

ReactDOM.render(
  (
    <Provider store={ store }>
      <Container />
    </Provider>
  ),
  document.querySelector('#container')
);
