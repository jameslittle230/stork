export class Entity {
  readonly name: string;
  readonly url: string;
  readonly config: Record<string, unknown>;
  constructor(name: string, url: string, config: Record<string, unknown>) {
    this.name = name;
    this.url = url;
    this.config = config;
  }

  registerIndex = jest.fn().mockResolvedValue({});
  attachToDom = jest.fn();
}
