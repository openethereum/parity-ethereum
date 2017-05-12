/* @flow */
import React, { Component } from 'react';

/** Components **/
import Search from '../../Components/UI/Search/Search';

/** Stylesheets **/
import styles from './Header.css';

// type Props = {|
//
// |}

// type State = {|
//
// |}

class Header extends Component {
  // props: Props;
  // state: State = {
  // };

  render () {
    return (
      <div className={ styles.Header }>
        {/* <Home />
        <Installed />
        <History /> */}
        <div id={ styles.headerSearch }>
          <Search />
        </div>
        Home

      </div>
    );
  }
}

export default Header;
