import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ContentClear from 'material-ui/svg-icons/content/clear';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import styles from './style.css';

export default class Tooltip extends Component {
  static propTypes = {
    title: PropTypes.string,
    text: PropTypes.string
  }

  state = {
    visible: true
  }

  render () {
    if (!this.state.visible) {
      return null;
    }

    return (
      <div className={ styles.box }>
        <div className={ styles.title }>
          { this.props.title }
        </div>
        <div className={ styles.text }>
          { this.props.text }
        </div>
        <div className={ styles.buttons }>
          <FlatButton
            icon={ <ContentClear /> }
            label='Skip'
            onTouchTap={ this.onClose } />
          <FlatButton
            icon={ <NavigationArrowForward /> }
            label='Next'
            onTouchTap={ this.onNext } />
        </div>
      </div>
    );
  }

  onClose = () => {
    this.setState({
      visible: false
    });
  }
}
