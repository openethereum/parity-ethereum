
import React, { Component, PropTypes } from 'react';
import IconButton from 'material-ui/IconButton';
import IconEventNote from 'material-ui/svg-icons/notification/event-note';

import styles from './Footer.css';

export default class Footer extends Component {

  render () {
    return (
      <footer { ...this._testInherit() }>
        <div className={ styles.footer }>
          <a href='http://ethcore.io'>ethcore.io</a>
          { this.renderLogIcon() }
          <span className={ styles.right }>
            Powered by: { this.props.version }
          </span>
        </div>
      </footer>
    );
  }

  renderLogIcon () {
    const { updateLogging, logging } = this.props;
    const isOffClass = !logging ? styles.off : '';

    const onClick = () => updateLogging(!logging);

    return (
      <IconButton
        { ...this._testInherit('log-button') }
        onClick={ onClick }
        tooltip='Toggle logging' tooltipPosition='top-left'
        className={ styles.logButton }
        >
        <IconEventNote className={ `${styles.logIcon} ${isOffClass}` } />
      </IconButton>
    );
  }

  static propTypes = {
    version: PropTypes.string.isRequired,
    logging: PropTypes.bool.isRequired,
    updateLogging: PropTypes.func.isRequired
  }

}
