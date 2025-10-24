import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import React, { useState } from "react";

type SidebarItem = {
    label: string;
    icon?: React.ReactNode;
    isActive?: boolean;
};

const sidebarItems: SidebarItem[] = [
    { label: "基本", isActive: false },
    { label: "字幕", isActive: true },
    { label: "输入", isActive: false },
    { label: "系统", isActive: false },
    { label: "关于", isActive: false },
];

export default function Settings() {
    const [open, setOpen] = useState(false);

    async function toggleCaption() {
        const win = await WebviewWindow.getByLabel("caption")

        if (win != null) {
            setOpen(false)
            await win.destroy()
            return
        }

        new WebviewWindow('caption', {
            url: '/index.html#caption',
            title: '字幕',
            width: 800,
            height: 200,
            transparent: true,
            decorations: false,
            resizable: true,
            acceptFirstMouse: true,
            closable: false,
            minimizable: false,
            maximizable: false,
            alwaysOnTop: true
        })

        setOpen(true)
    }

    async function transcribe() {
        await invoke<boolean>('transcribe')
    }

    return (
        <div className="flex h-screen bg-gray-100">
            <aside className="w-64 bg-white border-r flex flex-col p-4">
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
            
            <main className="flex-grow p-6 overflow-auto">
                <div className="space-y-4">
                    <section>
                        <h3 className="text-lg font-semibold mb-2">字幕</h3>
                        <button onClick={() => toggleCaption()} className="bg-blue-500 text-white px-2 py-2 rounded-md">
                            {open ? "关闭" : "打开"}
                        </button>
                    </section>
                    <section>
                        <h3 className="text-lg font-semibold mb-2">转录</h3>
                        <button onClick={() => transcribe()} className="bg-blue-500 text-white px-2 py-2 rounded-md">
                            转录
                        </button>
                    </section>
                </div>
            </main>
        </div>
    );
};