
import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';

class AccountsPage extends Component {

  render () {
    return (
      <div className='dapp-flex-content'>
        <main className='dapp-content'>
          <h1>Accounts</h1>
        </main>
      </div>
    );
  }

  static propTypes = {
    logger: PropTypes.object.isRequired,
    actions: PropTypes.object.isRequired,
    status: PropTypes.object.isRequired
  }
}

function mapStateToProps (state) {
  return state;
}

function mapDispatchToProps (dispatch) {
  return {};
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(AccountsPage);
