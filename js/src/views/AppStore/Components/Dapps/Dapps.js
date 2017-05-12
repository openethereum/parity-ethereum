/* @flow */
import React, { Component } from 'react';

/** Components **/
import FullApp from '../FullApp/FullApp';
import DappHeader from './DappHeader/DappHeader';

/** Stylesheets **/
import styles from './Dapps.css';

type Props = {|
  params: {
    appPath: string
  }
|}

// // type State = {|
// //
// // |}

class Dapps extends Component {
  props: Props;
  // state: State = {
  // };

  render () {
    const { appPath } = this.props.params;

    return (
      <div className={ styles.Dapps }>

        <DappHeader history={ 'history' } />
        <FullApp appId={ appPath } />

      </div>
    );
  }
}

export default Dapps;
