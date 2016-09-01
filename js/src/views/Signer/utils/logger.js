// this module disable logging on prod

const isLogging = process.env.LOGGING;

export default logger();

function logger () {
  return !isLogging ? prodLogger() : devLogger();
}

function prodLogger () {
  return {
    log: noop,
    info: noop,
    error: noop,
    warn: noop
  };
}

function devLogger () {
  return {
    log: console.log.bind(console),
    info: console.info.bind(console),
    error: console.error.bind(console),
    warn: console.warn.bind(console)
  };
}

function noop () {}
