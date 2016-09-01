import React, { Component, PropTypes } from 'react';
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';

import ToastrContainer from '../../components/ToastrContainer';
import { removeToast } from '../../actions/toastr';
import Header from '../../components/Header';

import styles from './Root.css';

// todo [adgo] - add animation wrap children
export class Root extends Component {

  static propTypes = {
    children: PropTypes.node.isRequired,
    toastr: PropTypes.shape({
      toasts: PropTypes.array.isRequired
    }).isRequired,
    actions: PropTypes.shape({
      removeToast: PropTypes.func.isRequired
    }).isRequired
  };

  render () {
    const { children, toastr, actions } = this.props;
    return (
      <div className={ styles.container }>
        <Header />
        <div className={ styles.mainContainer }>
          { children }
        </div>
        <ToastrContainer
          toasts={ toastr.toasts }
          actions={ actions }
        />
      </div>
    );
  }

}

function mapStateToProps (state) {
  return state;
}

function mapDispatchToProps (dispatch) {
  return {
    actions: bindActionCreators({ removeToast }, dispatch)
  };
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Root);
