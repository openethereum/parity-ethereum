import React, { Component, PropTypes } from 'react';
import AvPause from 'material-ui/svg-icons/av/pause';
import AvPlay from 'material-ui/svg-icons/av/play-arrow';
import AvReplay from 'material-ui/svg-icons/av/replay';

import { Container, ContainerTitle } from '../../../../ui';

import styles from './Debug.css';

export default class Debug extends Component {
  static propTypes = {
    actions: PropTypes.shape({
      removeDevLogs: PropTypes.func.isRequired,
      updateDevLogging: PropTypes.func.isRequired
    }).isRequired,
    nodeStatus: PropTypes.object.isRequired,
    statusDebug: PropTypes.shape({
      levels: PropTypes.string.isRequired,
      logging: PropTypes.bool.isRequired,
      logs: PropTypes.arrayOf(PropTypes.string).isRequired
    }).isRequired
  }

  render () {
    const { nodeStatus } = this.props;
    const { devLogs, devLogsLevels } = nodeStatus;

    if (!devLogs) {
      return null;
    }

    return (
      <Container>
        <ContainerTitle
          title='Node Logs' />
        { this.renderActions() }
        <h2 className={ styles.subheader }>
          { devLogsLevels || '-' }
        </h2>
        <div className={ styles.logs }>
          { this.renderLogs() }
        </div>
      </Container>
    );
  }

  renderLogs () {
    const { nodeStatus } = this.props;
    const { devLogs } = nodeStatus;

    return devLogs.map((log, idx) => (
      <pre className={ styles.log } key={ idx }>
        { log }
      </pre>
    ));
  }

  renderActions () {
    const toggleButton = this.props.statusDebug.logging
      ? <AvPause />
      : <AvPlay />;

    return (
      <div className={ styles.actions }>
        <a onClick={ this.toggle }>{ toggleButton }</a>
        <a onClick={ this.clear }><AvReplay /></a>
      </div>
    );
  }

  clear = () => {
    this.props.actions.removeDevLogs();
  }

  toggle = () => {
    this.props.actions.updateDevLogging(!this.props.statusDebug.logging);
  }
}
