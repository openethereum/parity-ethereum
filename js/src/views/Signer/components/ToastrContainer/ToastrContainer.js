
import React, { Component, PropTypes } from 'react';
import Paper from 'material-ui/Paper';

import styles from './ToastrContainer.css';

export default class ToastrContainer extends Component {

  static propTypes = {
    toasts: PropTypes.array.isRequired,
    actions: PropTypes.shape({
      removeToast: PropTypes.func.isRequired
    }).isRequired
  }

  render () {
    return (
      <div className={ styles.toastrContainer }>
        { this.renderToasts() }
      </div>
    );
  }

  renderToasts () {
    return this.props.toasts.map(t => {
      const removeToast = () => this.props.actions.removeToast(t.toastNo);

      return (
        <Paper
          className={ `${styles.toast} ${styles[t.type]}` }
          zDepth={ 2 }
          key={ t.toastNo }
          >
          <a className={ styles.remove } onClick={ removeToast }>
            &times;
          </a>
          <span className={ styles.msg }>{ t.msg }</span>
        </Paper>
      );
    });
  }

}
