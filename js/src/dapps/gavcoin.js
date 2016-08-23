import ReactDOM from 'react-dom';
import React from 'react';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

ReactDOM.render(
  <div>
    Welcome to GAVcoin
  </div>,
  document.querySelector('#container')
);
