import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';

import store from './registry/store'
import Container from './registry/Container';
import './registry.html';

ReactDOM.render(
  (
    <Provider store={ store }>
      <Container />
    </Provider>
  ),
  document.querySelector('#container')
);
