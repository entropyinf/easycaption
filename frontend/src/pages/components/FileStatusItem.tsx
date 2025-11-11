import { FileInfo } from "../../cmds/types";
import { downloadRequiredFile, stopDownloadRequiredFile } from "../../cmds";
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

interface FileStatusItemProps {
    file: FileInfo;
    modelDir: string;
}

interface DownloadProgress {
    file_name: string;
    size: number;
    position: number;
}

export default function FileStatusItem({ file, modelDir }: FileStatusItemProps) {
    const [downloadSize, setDownloadSize] = useState(0);
    const [isDownloading, setIsDownloading] = useState(false);

    const fileExisted = file.existed || downloadSize >= file.size;

    useEffect(() => {
        let unlisten: (() => void) | null = null;

        const setupListener = async () => {
            unlisten = await listen<DownloadProgress>('download_progress', (event) => {
                let progress = event.payload
                if (progress.file_name === file.name) {
                    setDownloadSize(progress.position);
                    if (progress.size > 0) {
                        setIsDownloading(progress.position < progress.size);
                    }
                }
            });
        };

        setupListener();

        return () => {
            if (unlisten) {
                unlisten();
            }
        };
    }, [file.name]);

    const formatFileSize = (bytes: number): string => {
        if (bytes === 0) return '0 B';

        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));

        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    };

    const handleDownload = async () => {
        if (file.existed) return;

        if (isDownloading) {
            try {
                await stopDownloadRequiredFile(file.name);
                setIsDownloading(false);
            } catch (error) {
                console.error("Failed to stop download:", error);
            }
        } else {
            try {
                setIsDownloading(true);
                await downloadRequiredFile(modelDir, file.name);
            } catch (error) {
                console.error("Failed to start download:", error);
                setIsDownloading(false);
            }
        }
    };

    const size = formatFileSize(file.size);
    const downloadedSize = formatFileSize(downloadSize);
    const progressPercentage = file.size > 0 ? Math.min(100, Math.round((downloadSize / file.size) * 100)) : 0;

    const renderActionButton = () => {
        if (fileExisted) {
            return (
                <span className="inline-flex items-center ml-2 px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 whitespace-nowrap">
                    已下载
                </span>
            );
        }

        let buttonText = '下载';
        let buttonClass = 'bg-blue-500 hover:bg-blue-600';

        if (downloadSize > 0) {
            buttonText = '继续';
            buttonClass = 'bg-yellow-500 hover:bg-yellow-600';
        }

        if (isDownloading) {
            buttonText = '暂停';
            buttonClass = 'bg-red-500 hover:bg-red-600';
        }

        return (
            <button
                onClick={handleDownload}
                className={`inline-flex items-center ml-2 px-2 py-0.5 rounded-full text-xs font-medium text-white whitespace-nowrap transition-colors duration-200 ${buttonClass}`}
            >
                {buttonText}
            </button>
        );
    };

    return (
        <div className="px-3 py-2 flex justify-between rounded-md bg-gray-50 hover:bg-gray-100 transition-colors duration-200 w-full items-center">
            <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between">
                    <span className="font-medium text-gray-900 truncate text-sm">{file.name}</span>
                    <div className="flex items-center text-xs text-gray-500 ml-2">
                        <span>{size}</span>
                    </div>
                </div>

                {(isDownloading || downloadSize > 0) && (
                    <div className="mt-1.5">
                        <div className="flex justify-between text-xs text-gray-500 mb-1">
                            <span>{downloadedSize} / {size}</span>
                            <span>{progressPercentage}%</span>
                        </div>
                        <div className="w-full bg-gray-200 rounded-full h-1.5">
                            <div
                                className="bg-blue-500 h-1.5 rounded-full transition-all duration-300 ease-out"
                                style={{ width: `${progressPercentage}%` }}
                            ></div>
                        </div>
                    </div>
                )}
            </div>

            {renderActionButton()}
        </div>
    );
}