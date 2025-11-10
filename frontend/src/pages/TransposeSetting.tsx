import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { currentMonitor } from "@tauri-apps/api/window";
import { produce, type WritableDraft } from "immer";
import { useEffect, useState } from "react";
import { checkRequiredFiles, getDevices, getTranscribeConfig, updateTranscribeConfig } from "../cmds/index.ts"; // 修改这一行
import { FileInfo, TransposeConfig } from "../cmds/types.ts";
import FileStatusItem from "./components/FileStatusItem.tsx";
import NumberInput from "./components/NumberInput.tsx";
import SectionCard from "./components/SectionCard.tsx";
import SelectInput from "./components/SelectInput.tsx";
import SettingItem from "./components/SettingItem.tsx";
import TextInput from "./components/TextInput.tsx";


type AudioDevice = {
    host: string;
    device: string;
};

export default function TransposeSetting() {
    const [captionWindowVisiable, setCaptionWindowVisiable] = useState(false);
    const [config, setConfig] = useState<TransposeConfig | null>(null);
    const [inputs, setInputs] = useState<AudioDevice[]>([]);
    const [requiredFiles, setRequiredFiles] = useState<FileInfo[]>([]);

    useEffect(() => {
        fetchConfig();
        fetchDevices();
    }, []);

    async function fetchConfig() {
        const new_config = await getTranscribeConfig()
        if (new_config?.model_config.model_dir) {
            fetchCheckRequiredFiles(new_config.model_config.model_dir);
        }
        setConfig(new_config);
    }

    async function fetchDevices() {
        setInputs(await getDevices());
    }
    async function fetchCheckRequiredFiles(model_dir: string) {
        setRequiredFiles(await checkRequiredFiles(model_dir));
    }

    async function toggleCaption() {
        const WINDOW_NAME = "caption";

        const win = await WebviewWindow.getByLabel(WINDOW_NAME);

        if (win != null) {
            await win.destroy();
            setCaptionWindowVisiable(false);
            return;
        }

        const monitor = await currentMonitor();
        if (monitor) {
            console.log("monitor", JSON.stringify(monitor))
            const width = 800
            const scaleFactor = monitor.scaleFactor || 1;
            const x = monitor.size.width / scaleFactor / 2 - width / 2;
            const y = monitor.size.height / scaleFactor * 0.8;
            new WebviewWindow(WINDOW_NAME, {
                url: '/index.html#caption',
                title: '字幕',
                width: width,
                height: 200,
                x: x,
                y: y,
                transparent: true,
                decorations: false,
                resizable: true,
                acceptFirstMouse: true,
                closable: false,
                minimizable: false,
                maximizable: false,
                alwaysOnTop: true,
                visible: true
            })
            setCaptionWindowVisiable(true);
        }
    }

    const updateConfig = async (fn: (d: WritableDraft<TransposeConfig>) => void) => {
        if (!config) return;
        const new_config = produce(config, fn);
        await updateTranscribeConfig(new_config);
        await fetchConfig();
    };

    if (!config) return <div>Loading...</div>;

    return (
        <>
            <div className="space-y-2">
                <SectionCard title="输入">
                    <SelectInput
                        label="输入设备"
                        value={`${config.input_host}-${config.input_device}`}
                        onChange={(value) => {
                            const [host, ...deviceParts] = value.split('-');
                            const device = deviceParts.join('-');
                            updateConfig(draft => {
                                draft.input_device = device;
                                draft.input_host = host;
                            });
                        }}
                        options={inputs.map((d) => ({
                            value: `${d.host}-${d.device}`,
                            label: `${d.host} - ${d.device}`
                        }))}
                    />
                </SectionCard>


                <SectionCard title="基础">
                    <SettingItem
                        label="开启字幕"
                        description="开启字幕窗口"
                        checked={captionWindowVisiable}
                        onChange={toggleCaption}
                    />

                    <SettingItem
                        label="启用转录"
                        description="是否启用语音转文字功能"
                        checked={config.enable}
                        onChange={() => updateConfig(draft => {
                            draft.enable = !draft.enable;
                        })}
                    />



                    <SettingItem
                        label="使用GPU"
                        description="是否使用GPU (如果有CUDA 或者 Metal)"
                        checked={config.model_config.use_gpu}
                        onChange={() => updateConfig(draft => {
                            draft.model_config.use_gpu = !draft.model_config.use_gpu;
                        })}
                    />

                </SectionCard>



                <SectionCard title="实时性">
                    <SettingItem
                        label="开启"
                        description="提升实时性"
                        checked={config.realtime}
                        onChange={() => updateConfig(draft => {
                            draft.realtime = !draft.realtime;
                        })}
                    />
                    <NumberInput
                        label="间隔"
                        value={config.realtime_rate}
                        onChange={(value) => updateConfig(draft => {
                            draft.realtime_rate = value || 1000;
                        })}
                        placeholder="采样率"
                    />
                </SectionCard>


                <SectionCard title="模型">
                    <TextInput
                        label="模型目录"
                        value={config.model_config.model_dir}
                        onChange={(value) => updateConfig(draft => {
                            if (draft.model_config.model_dir === value) return;
                            draft.model_config.model_dir = value;
                            fetchCheckRequiredFiles(value)
                        })}
                        placeholder="模型缓存路径"
                        disable={config.enable}
                    />
                    <label className="block text-sm font-medium mb-1">所需文件</label>
                    <div>
                        {requiredFiles.map((file, index) => (
                            <div className="m-1">
                                <FileStatusItem
                                    key={index}
                                    file={file}
                                    modelDir={config.model_config.model_dir}
                                />
                            </div>
                        ))}
                    </div>
                </SectionCard>


                <SectionCard title="VAD">
                    <NumberInput
                        label="采样率 (Hz)"
                        value={config.model_config.vad.sample_rate}
                        onChange={(value) => updateConfig(draft => {
                            if (draft.model_config.vad) {
                                draft.model_config.vad.sample_rate = value || 0;
                            }
                        })}
                        placeholder="采样率"
                    />

                    <NumberInput
                        label="语音检测阈值"
                        value={config.model_config.vad.speech_threshold}
                        onChange={(value) => updateConfig(draft => {
                            if (draft.model_config.vad) {
                                draft.model_config.vad.speech_threshold = value || 0;
                            }
                        })}
                        placeholder="语音检测阈值"
                        step="0.01"
                    />

                    <NumberInput
                        label="最大静音时长 (ms)"
                        value={config.model_config.vad.silence_max_ms}
                        onChange={(value) => updateConfig(draft => {
                            if (draft.model_config.vad) {
                                draft.model_config.vad.silence_max_ms = value || 0;
                            }
                        })}
                        placeholder="最大静音时长"
                    />

                    <NumberInput
                        label="最小语音时长 (ms)"
                        value={config.model_config.vad.speech_min_ms}
                        onChange={(value) => updateConfig(draft => {
                            if (draft.model_config.vad) {
                                draft.model_config.vad.speech_min_ms = value || 0;
                            }
                        })}
                        placeholder="最小语音时长"
                    />

                    <NumberInput
                        label="平均语音时长 (ms)"
                        value={config.model_config.vad.speech_avg_ms}
                        onChange={(value) => updateConfig(draft => {
                            if (draft.model_config.vad) {
                                draft.model_config.vad.speech_avg_ms = value || 0;
                            }
                        })}
                        placeholder="平均语音时长"
                    />

                    <NumberInput
                        label="静音衰减因子"
                        value={config.model_config.vad.silence_attenuation_factor}
                        onChange={(value) => updateConfig(draft => {
                            if (draft.model_config.vad) {
                                draft.model_config.vad.silence_attenuation_factor = value || 0;
                            }
                        })}
                        placeholder="静音衰减因子"
                        step="0.01"
                    />
                </SectionCard>
            </div>
        </>
    );
};