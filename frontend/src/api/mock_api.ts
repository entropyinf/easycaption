
import { Api, Config } from "./api";

export class MockApi implements Api {
    config_get(): Promise<Config> {
        return Promise.resolve({
            no_vac: false,
            no_vad: false,
            never_fire: false,
            confidence_validation: false,
            diarization: false,
            punctuation_split: false,
            no_transcription: false,
        });
    }
    config_set(config: Config): Promise<void> {
        throw new Error("Method not implemented.");
    }
}
