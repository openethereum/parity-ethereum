/* @flow */
/* Copyright 2015-2017 Parity Technologies (UK) Ltd.
/* This file is part of Parity.
/*
/* Parity is free software: you can redistribute it and/or modify
/* it under the terms of the GNU General Public License as published by
/* the Free Software Foundation, either version 3 of the License, or
/* (at your option) any later version.
/*
/* Parity is distributed in the hope that it will be useful,
/* but WITHOUT ANY WARRANTY; without even the implied warranty of
/* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
/* GNU General Public License for more details.
/*
/* You should have received a copy of the GNU General Public License
/* along with Parity.  If not, see <http://www.gnu.org/licenses/>.
*/

import React, { Component } from 'react';

/** Components **/
import MedApp from '../../MedApp/MedApp';

/** Stylesheets **/
import styles from './Featured.css';

class Featured extends Component {
  render () {
    return (
      <div className={ styles.Featured }>

        <div className={ styles.featuredText }>
          Featured
        </div>

        <div className={ styles.dappContent }>
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
