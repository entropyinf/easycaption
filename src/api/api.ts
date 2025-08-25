export interface Api {
    config_get(): Promise<Config>;
    config_set(config: Config): Promise<void>;
}

export type Config = {
};