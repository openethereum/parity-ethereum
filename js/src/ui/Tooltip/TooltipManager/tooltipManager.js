export default class TooltipManager {
  constructor () {
    this.currentId = 0;
    this.updateCallbacks = [];
  }

  register (updateCallback) {
    this.updateCallbacks.push(updateCallback);
    this.update();

    return this.updateCallbacks.length - 1;
  }

  update () {
    this.updateCallbacks.forEach((cb) => {
      cb(this.currentId, this.updateCallbacks.length - 1);
    });
  }

  next () {
    this.currentId++;
    this.update();
  }

  close () {
    this.currentId = -1;
    this.update();
  }
}
