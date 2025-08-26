import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export default function Caption() {
    const [caption, setCaption] = useState('')

    useEffect(() => {
        try {
            listen<TranscriptionStatus>('caption', (event) => {
                try {
                    const lines = event.payload.lines.map(line => line.text).join("\n")
                    setCaption(lines)
                } catch (error) {
                    console.log(error)
                }
            });
        } catch (error) {
            console.log(error)
        }
    }, [])

    return (
        <div className="h-full bg-black/50 text-white p-4 rounded-md flex flex-col absolute bottom-0 left-0 right-0 select-none" >
            <h1 className="text-3xl font-medium text-center select-none" data-tauri-drag-region>
                {caption}
            </h1>
        </div>
    )

}

/**
 * 转录行数据接口
 */
type TranscriptionLine = {
    /**
     * 说话者ID
     * - 1: 表示正常说话者
     * - -2: 表示无说话内容
     */
    speaker: number;

    /**
     * 转录文本内容
     */
    text: string;

    /**
     * 开始时间 (格式: h:mm:ss)
     */
    beg: string;

    /**
     * 结束时间 (格式: h:mm:ss)
     */
    end: string;

    /**
     * 持续时间 (秒)
     */
    diff: number;
}

/**
 * 转录状态接口
 */
type TranscriptionStatus = {
    /**
     * 当前状态
     */
    status: string;

    /**
     * 转录行数组
     */
    lines: TranscriptionLine[];

    /**
     * 转录缓冲区内容
     */
    buffer_transcription: string;

    /**
     * 说话人识别缓冲区内容
     */
    buffer_diarization: string;

    /**
     * 转录剩余时间
     */
    remaining_time_transcription: number;

    /**
     * 说话人识别剩余时间
     */
    remaining_time_diarization: number;
}