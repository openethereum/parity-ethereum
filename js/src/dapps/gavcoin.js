import ReactDOM from 'react-dom';
import React from 'react';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import Application from './gavcoin/Application';

import './style.css';

ReactDOM.render(
  <Application />,
  document.querySelector('#container')
);
