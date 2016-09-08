
export function scrollTo (element, to, duration) {
  let start = element.scrollTop;
  let change = to - start;
  let increment = 50;

  let animateScroll = elapsedTime => {
    elapsedTime += increment;
    let position = easeInOut(elapsedTime, start, change, duration);
    element.scrollTop = position;
    if (elapsedTime < duration) {
      setTimeout(() => {
        // stop if user scrolled
        if (element.scrollTop !== parseInt(position, 10)) {
          return;
        }
        animateScroll(elapsedTime);
      }, increment);
    }
  };

  animateScroll(0);
}

export function easeInOut (currentTime, start, change, duration) {
  currentTime /= duration / 2;
  if (currentTime < 1) {
    return change / 2 * currentTime * currentTime + start;
  }
  currentTime -= 1;
  return -change / 2 * (currentTime * (currentTime - 2) - 1) + start;
}
