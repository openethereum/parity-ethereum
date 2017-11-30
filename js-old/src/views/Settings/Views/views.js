// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { Checkbox } from 'material-ui';

import { Container } from '~/ui';

import { toggleView } from '../actions';

import layout from '../layout.css';
import styles from './views.css';

class Views extends Component {
  static propTypes = {
    settings: PropTypes.object.isRequired,
    toggleView: PropTypes.func.isRequired,
    availability: PropTypes.string.isRequired
  }

  render () {
    return (
      <Container
        title={
          <FormattedMessage id='settings.views.label' />
        }
      >
        <div className={ layout.layout }>
          <div className={ layout.overview }>
            <div>
              <FormattedMessage
                id='settings.views.overview_0'
                defaultMessage='Manage the available application views using only the parts of the application applicable to you.'
              />
            </div>
            <div>
              <FormattedMessage
                id='settings.views.overview_1'
                defaultMessage='Are you an end-user? The defaults are setup for both beginner and advanced users alike.'
              />
            </div>
            <div>
              <FormattedMessage
                id='settings.views.overview_2'
                defaultMessage='Are you a developer? Add some features to manage contracts and interact with application deployments.'
              />
            </div>
            <div>
              <FormattedMessage
                id='settings.views.overview_3'
                defaultMessage='Are you a miner or run a large-scale node? Add the features to give you all the information needed to watch the node operation.'
              />
            </div>
          </div>
          <div className={ layout.details }>
            {
              this.renderView('accounts',
                <FormattedMessage
                  id='settings.views.accounts.label'
                />,
                <FormattedMessage
                  id='settings.views.accounts.description'
                  defaultMessage='A list of all the accounts associated with and imported into this Parity instance. Send transactions, receive incoming values, manage your balances and fund your accounts.'
                />
              )
            }
            {
              this.renderView('addresses',
                <FormattedMessage
                  id='settings.views.addresses.label'
                />,
                <FormattedMessage
                  id='settings.views.addresses.description'
                  defaultMessage='A list of all contacts and address book entries managed by this Parity instance. Watch accounts and have the details available at the click of a button when transacting.'
                />
              )
            }
            {
              this.renderView('contracts',
                <FormattedMessage
                  id='settings.views.contracts.label'
                />,
                <FormattedMessage
                  id='settings.views.contracts.description'
                  defaultMessage='Watch and interact with specific contracts that have been deployed on the network. This is a more technically-focused environment, specifically for advanced users that understand the inner working of certain contracts.'
                />
              )
            }
            {
              this.renderView('settings',
                <FormattedMessage
                  id='settings.views.settings.label'
                />,
                <FormattedMessage
                  id='settings.views.settings.description'
                  defaultMessage='This view. Allows you to customize the application in term of options, operation and look and feel.'
                />
              )
            }
          </div>
        </div>
      </Container>
    );
  }

  renderViews () {
    const { settings } = this.props;

    return Object.keys(settings.views).map((id) => {
      const description = <FormattedMessage id={ `settings.views.${id}.description` } />;
      const label = <FormattedMessage id={ `settings.views.${id}.label` } />;

      this.renderView(id, label, description);
    });
  }

  renderView = (id, label, description) => {
    const { settings, toggleView } = this.props;

    const toggle = () => toggleView(id);
    const view = settings.views[id];

    if (view.onlyPersonal && this.props.availability !== 'personal') {
      return null;
    }

    return (
      <div className={ styles.view } key={ id }>
        <Checkbox
          disabled={ view.fixed }
          label={
            <div className={ styles.header }>
              <div className={ styles.labelicon }>
                { view.icon }
              </div>
              <div className={ styles.label }>
                { label }
              </div>
            </div>
          }
          onCheck={ toggle }
          checked={ view.active }
          value={ view.active }
        />
        <div className={ styles.info }>
          { description }
        </div>
      </div>
    );
  }
}

function mapStateToProps (state) {
  const { settings } = state;
  const { availability = 'unknown' } = state.nodeStatus.nodeKind || {};

  return { settings, availability };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({ toggleView }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Views);
