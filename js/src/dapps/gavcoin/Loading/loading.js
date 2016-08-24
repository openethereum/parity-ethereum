import React, { Component } from 'react';

import { CircularProgress } from 'material-ui';

export default class Loading extends Component {
  render () {
    return (
      <div className='loading'>
        <CircularProgress size={ 2 } />
      </div>
    );
  }
}
