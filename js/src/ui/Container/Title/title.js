import React, { Component, PropTypes } from 'react';

import { CardTitle } from 'material-ui/Card';

import styles from './title.css';

const TITLE_STYLE = { textTransform: 'uppercase', padding: 0 };

export default class Title extends Component {
  static propTypes = {
    title: PropTypes.oneOfType([
      PropTypes.string, PropTypes.node
    ]),
    byline: PropTypes.string
  }

  state = {
    name: 'Unnamed'
  }

  render () {
    return (
      <div>
        <CardTitle
          style={ TITLE_STYLE }
          title={ this.props.title } />
        <div className={ styles.byline }>
          { this.props.byline }
        </div>
      </div>
    );
  }
}
