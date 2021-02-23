class StorkError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "StorkError";
  }
}

export default StorkError;
