
import React, { useState } from "react";
import NotificationContainer from "./components/Notification.tsx";
import TransposeSetting from "./TransposeSetting.tsx";


type SidebarItem = {
    label: string;
    icon?: React.ReactNode;
    component: React.ReactNode;
};

const menus: Record<string, SidebarItem> = {
    "transpose-setting": {
        label: "配置",
        icon: <i className="fas fa-cog"></i>,
        component: <TransposeSetting />,
    },
}


export default function Layout() {
    const [activeMenu, setActiveMenu] = useState<string | null>("transpose-setting");
    return <div className="flex h-screen bg-gray-100 select-none">
        <NotificationContainer />
        <aside className="w-48 bg-white flex flex-col p-4">
            <div className="mb-6">
                <h2 className="text-xl font-bold" data-tauri-drag-region>Easy Caption</h2>
            </div>
            <nav className="flex-grow">
                <ul>
                    {
                        Object.entries(menus).map(([key, item]) => (
                            <li
                                key={key}
                                className={`py-2 px-3 rounded-md cursor-pointer ${key === activeMenu
                                    ? "bg-blue-500 text-white"
                                    : "hover:bg-gray-100"
                                    }`}
                                onClick={() => setActiveMenu(key)}
                            >
                                {item.icon}
                                {item.label}
                            </li>
                        ))
                    }
                </ul>
            </nav>
        </aside>

        <main className="flex-grow p-2 overflow-auto">
            {activeMenu && menus[activeMenu].component}
        </main>
    </div>

}
