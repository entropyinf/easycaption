import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { currentMonitor } from "@tauri-apps/api/window";
import React, { useEffect, useState } from "react";
import { getTranscribeConfig, updateTranscribeConfig, getDevices, checkRequiredFiles } from "../cmds"; // 修改这一行
import { FileInfo, TransposeConfig } from "../cmds/types";
import { produce, type WritableDraft } from "immer";
import SettingItem from "./components/SettingItem";
import SectionCard from "./components/SectionCard";
import TextInput from "./components/TextInput";
import NumberInput from "./components/NumberInput";
import SelectInput from "./components/SelectInput";
import FileStatusItem from "./components/FileStatusItem";
import NotificationContainer from "./components/Notification.tsx";

type SidebarItem = {
    label: string;
    icon?: React.ReactNode;
    isActive?: boolean;
};

type AudioDevice = {
    host: string;
    device: string;
};

const sidebarItems: SidebarItem[] = [
    { label: "配置", isActive: true },
    { label: "关于", isActive: false },
];

export default function Settings() {
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
                height: 100,
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

    const renderConfigForm = () => {
        if (!config) return <div>Loading...</div>;

        return (
            <div className="space-y-2">
                <SectionCard title="基础配置">
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
                        description="是否使用GPU(如果有的话)"
                        checked={config.model_config.use_gpu}
                        onChange={() => updateConfig(draft => {
                            draft.model_config.use_gpu = !draft.model_config.use_gpu;
                        })}
                    />

                </SectionCard>


                <SectionCard title="输入配置">
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


                <SectionCard title="模型配置">
                    <TextInput
                        label="模型目录"
                        value={config.model_config.model_dir}
                        onChange={(value) => updateConfig(draft => {
                            if (draft.model_config.model_dir === value) return;
                            draft.model_config.model_dir = value;
                            fetchCheckRequiredFiles(value)
                        })}
                        placeholder="模型缓存路径"
                    />
                    <label className="block text-sm font-medium mb-1">所需文件</label>
                    {requiredFiles.map((file, index) => (
                        <FileStatusItem
                            key={index}
                            file={file}
                            modelDir={config.model_config.model_dir}
                        />
                    ))}
                </SectionCard>


                <SectionCard title="VAD 配置">
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
        );
    };

    return (
        <div className="flex h-screen bg-gray-100 select-none">
            <NotificationContainer />
            <aside className="w-48 bg-white flex flex-col p-4">
                <div className="mb-6">
                    <h2 className="text-xl font-bold" data-tauri-drag-region>EasyCaption</h2>
                </div>
                <nav className="flex-grow">
                    <ul>
                        {sidebarItems.map((item, index) => (
                            <li
                                key={index}
                                className={`py-2 px-3 rounded-md cursor-pointer ${item.isActive
                                    ? "bg-blue-500 text-white"
                                    : "hover:bg-gray-100"
                                    }`}
                            >
                                {item.label}
                            </li>
                        ))}
                    </ul>
                </nav>
            </aside>

            <main className="flex-grow p-2 overflow-auto">
                {renderConfigForm()}
            </main>
        </div>
    );
};