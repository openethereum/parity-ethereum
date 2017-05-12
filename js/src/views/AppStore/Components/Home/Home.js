/* @flow */
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
