import React from 'react';
import ToggleSwitch from './ToggleSwitch';

interface SettingItemProps {
    label: string;
    description: string;
    checked: boolean;
    onChange: () => void;
}

const SettingItem: React.FC<SettingItemProps> = ({ label, description, checked, onChange }) => {
    return (
        <div className="flex items-center justify-between py-1.5">
            <div className="flex-1">
                <label className="block text-sm font-medium text-gray-900">{label}</label>
                <p className="text-xs text-gray-500 mt-0.5">{description}</p>
            </div>
            <ToggleSwitch checked={checked} onChange={onChange} />
        </div>
    );
};

export default SettingItem;