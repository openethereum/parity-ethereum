/* @flow */
import React, { Component } from 'react';

/** Components **/
import Search from '../../Components/UI/Search/Search';

/** Stylesheets **/
import './Header.css';

type Props = {|

|}

// type State = {|
//
// |}

class Header extends Component {
  props: Props;
  // state: State = {
  // };

  render() {
    return (
      <div className="Header">
        {/*<Home />
        <Installed />
        <History />*/}
        <div id="header-search">
          <Search />
        </div>
        Home

      </div>
    );
  }
}

export default Header;
