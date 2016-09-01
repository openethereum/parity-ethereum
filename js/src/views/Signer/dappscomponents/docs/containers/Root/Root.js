import React, { Component, PropTypes } from 'react';
import AnimateChildren from '../../../AnimateChildren';
import Header from '../../components/Header';

import styles from './Root.css';

export default class Root extends Component {

  static propTypes = {
    children: PropTypes.node.isRequired,
    location: PropTypes.shape({
      pathname: PropTypes.string.isRequired
    }).isRequired
  };

  render () {
    const { location, children } = this.props;
    return (
      <div className={ styles.container }>
        <Header />
        <AnimateChildren absolute isView pathname={ location.pathname }>
          { children }
        </AnimateChildren>
      </div>
    );
  }

}
