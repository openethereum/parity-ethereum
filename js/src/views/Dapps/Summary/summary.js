import React, { Component, PropTypes } from 'react';

import Container, { Title } from '../../../ui/Container';
import IdentityIcon from '../../../ui/IdentityIcon';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    app: PropTypes.object.isRequired,
    children: PropTypes.node
  }

  render () {
    const { app } = this.props;

    if (!app) {
      return null;
    }

    return (
      <Container>
        <IdentityIcon
          address={ app.address } />
        <Title
          title={ <a href={ app.url }>{ app.name }</a> }
          byline={ app.description } />
        { this.props.children }
      </Container>
    );
  }
}
