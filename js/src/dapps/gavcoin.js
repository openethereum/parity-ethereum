import ReactDOM from 'react-dom';
import React from 'react';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import Application from './gavcoin/Application';

import './style.css';
import './gavcoin.html';

ReactDOM.render(
  <Application />,
  document.querySelector('#container')
);
