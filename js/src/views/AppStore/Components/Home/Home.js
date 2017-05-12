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
import Catagories from './Catagories/Catagories';
import Featured from './Featured/Featured';
import Headlines from './Headlines/Headlines';
import New from './New/New';

/** Stylesheets **/
import styles from './Home.css';

class Home extends Component {
  render () {
    return (
      <div className={ styles.Home }>

        <div className='col-md-12' id={ styles.homeHeadlines }>
          <div className='col-md-12'>
            <Headlines />
          </div>
        </div>

        <div className='col-md-12' id={ styles.homeMain }>
          <div className='col-md-3' id={ styles.homeCatagories }>
            <Catagories />
          </div>

          <div className='col-md-9' id={ styles.homeFeaturedNew }>
            <Featured />
            <New />
          </div>
        </div>

      </div>
    );
  }
}

export default Home;
