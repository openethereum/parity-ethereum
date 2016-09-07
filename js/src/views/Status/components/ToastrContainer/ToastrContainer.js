
import React, { Component, PropTypes } from 'react';
import Paper from 'material-ui/Paper';

import styles from './ToastrContainer.css';

export default class ToastrContainer extends Component {

  render () {
    return (
      <div className={ styles.toastrContainer }>
        { this.renderToasts() }
      </div>
    );
  }

  renderToasts () {
    return this.props.statusToastr.toasts.map(t => {
      const removeToast = () => this.props.actions.removeToast(t.toastNo);

      return (
        <Paper
          className={ `${styles.toast} ${styles[t.type]}` }
          zDepth={ 2 }
          key={ t.toastNo }
          { ...this._test(`toast-${t.toastNo}`) }
          >
          <a className={ styles.remove } onClick={ removeToast }>
            <i className='icon-trash'></i>
          </a>
          <span className={ styles.msg }>{ t.msg }</span>
        </Paper>
      );
    });
  }

  static propTypes = {
    statusToastr: PropTypes.shape({
      toasts: PropTypes.array.isRequired
    }).isRequired,
    actions: PropTypes.shape({
      removeToast: PropTypes.func.isRequired
    }).isRequired
  }

}
