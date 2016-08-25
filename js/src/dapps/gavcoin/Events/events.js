import React, { Component, PropTypes } from 'react';

import EventBuyin from './EventBuyin';
import EventNewTranch from './EventNewTranch';

export default class Events extends Component {
  static contextTypes = {
    instance: PropTypes.object
  }

  state = {
    allEvents: [],
    minedEvents: [],
    pendingEvents: []
  }

  componentDidMount () {
    this.setupFilters();
  }

  render () {
    return (
      <div className='events'>
        { this.renderEvents() }
      </div>
    );
  }

  renderEvents () {
    const { allEvents } = this.state;

    if (!allEvents.length) {
      return null;
    }

    return allEvents
      .map((event) => {
        switch (event.type) {
          case 'Buyin':
            return <EventBuyin key={ event.key } event={ event } />;
          case 'NewTranch':
            return <EventNewTranch key={ event.key } event={ event } />;
        }
      });
  }

  setupFilters () {
    const { instance } = this.context;
    let key = 0;

    ['Approval', 'Buyin', 'Refund', 'Transfer', 'NewTranch'].forEach((eventName) => {
      const options = {
        fromBlock: 0,
        toBlock: 'pending'
      };

      const logToEvent = (log) => {
        const { blockNumber, logIndex, transactionHash, transactionIndex, params, type } = log;

        return {
          type: eventName,
          state: type,
          blockNumber,
          logIndex,
          transactionHash,
          transactionIndex,
          params,
          key: ++key
        };
      };

      const sortEvents = (a, b) => b.blockNumber.cmp(a.blockNumber) || b.logIndex.cmp(a.logIndex);

      instance[eventName].subscribe(options, (logs) => {
        if (!logs.length) {
          return;
        }

        console.log(logs);

        const minedEvents = this.state.minedEvents
          .concat(logs.filter((log) => log.type === 'mined').map(logToEvent))
          .sort(sortEvents);
        const pendingEvents = this.state.pendingEvents
          .filter((event) => {
            return !logs.find((log) => {
              return (log.type === 'mined') && (log.transactionHash === event.transactionHash);
            });
          })
          .reverse()
          .concat(logs.filter((log) => log.type === 'pending').map(logToEvent))
          .reverse();
        const allEvents = pendingEvents.concat(minedEvents);

        this.setState({
          allEvents,
          minedEvents,
          pendingEvents
        });
      });
    });
  }
}
