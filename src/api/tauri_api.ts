import { Api, Config } from "./api";

export class TauriApi implements Api {
    config_get(): Promise<Config> {
        throw new Error("Method not implemented.");
    }
    config_set(config: Config): Promise<void> {
        throw new Error("Method not implemented.");
    }
}