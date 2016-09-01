
import React, { Component, PropTypes } from 'react';

import Toast from '../Toast';
import styles from './Toastr.css';

export default class Toastr extends Component {

  static propTypes = {
    className: PropTypes.string,
    toasts: PropTypes.arrayOf(
      PropTypes.shape({
        id: PropTypes.number.isRequired,
        type: PropTypes.string.isRequired,
        msg: PropTypes.string.isRequired
      })
    ).isRequired,
    onRemoveToast: PropTypes.func.isRequired
  }

  render () {
    const { className } = this.props;
    return (
      <div className={ `${styles.container} ${className}` }>
        { this.renderToasts() }
      </div>
    );
  }

  renderToasts () {
    const { onRemoveToast } = this.props;
    return this.props.toasts.map(t => (
      <Toast { ...t } onRemoveToast={ onRemoveToast } key={ t.id } />
    ));
  }

}
