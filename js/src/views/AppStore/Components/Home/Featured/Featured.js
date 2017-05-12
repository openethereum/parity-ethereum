/* @flow */
import React, { Component } from 'react';

/** Components **/
import MedApp from '../../MedApp/MedApp';

/** Stylesheets **/
import './Featured.css';

class Featured extends Component {
  render () {
    return (
      <div className='Featured'>

        <div className='featured-text'>
          Featured
        </div>

        <div className='dapp-content'>
          <MedApp hash='0x264D14eAbB717Ea34F1540757e364727fdC75eA4' />
          <MedApp hash='0x264D14eAbB717Ea34F1540757e364727fdC75eA4' />
          <MedApp hash='0x264D14eAbB717Ea34F1540757e364727fdC75eA4' />
          <MedApp hash='0x264D14eAbB717Ea34F1540757e364727fdC75eA4' />
          <MedApp hash='0x264D14eAbB717Ea34F1540757e364727fdC75eA4' />
        </div>

      </div>
    );
  }
}

export default Featured;
