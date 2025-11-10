
/**
 * Configuration for transpose functionality
 */
export type TransposeConfig = {
    /**
     * Whether transpose is enabled
     */
    enable: boolean;

    /**
     * Input host configuration
     */
    input_host: string;

    /**
     * Input device configuration
     */
    input_device: string;


    /**
     * Whether to transpose in real-time
     */
    realtime: boolean;

    /**
     * Real-time rate for transpose
     */
    realtime_rate: number;

    /**
     * Model configuration for SenseVoiceSmall
     */
    model_config: SenseVoiceSmallConfig;
};

/**
 * Configuration for SenseVoiceSmall model
 */
export type SenseVoiceSmallConfig = {

    /**
     * Path to model cache dir
     */
    model_dir: string;

    /**
     * Voice Activity Detection configuration
     */
    vad: VadConfig;

    /**
     * Resample configuration as [from, to] sample rates
     */
    resample?: [number, number];

    /**
     * Whether to use GPU for inference
     */
    use_gpu: boolean;
};

/**
 * Configuration parameters for Voice Activity Detection
 */
export type VadConfig = {
    /**
    * Sample rate in Hz (e.g., 16000)
    */
    sample_rate: number;

    /**
     * Threshold for speech detection, values above this are considered voice activity
     */
    speech_threshold: number;

    /**
     * Maximum silence duration in milliseconds, exceeding this ends a speech segment
     */
    silence_max_ms: number;

    /**
     * Minimum speech duration in milliseconds, segments shorter than this are ignored
     */
    speech_min_ms: number;

    /**
     * Average speech duration in milliseconds, used to dynamically adjust silence detection parameters
     */
    speech_avg_ms: number;

    /**
     * Factor to adjust silence detection sensitivity after long speech segments
     */
    silence_attenuation_factor: number;
};


export type FileInfo = {
    name: string;
    path: string;
    size: number;
    sha256: string;
    absolute_path: string;
    downloaded_size: number;
    existed: boolean;
};