import ReactDOM from 'react-dom';
import React from 'react';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import Application from './tokenreg/Application';

ReactDOM.render(
  <Application />,
  document.querySelector('#container')
);
