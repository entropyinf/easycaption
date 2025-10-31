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
        <div className="flex items-center justify-between">
            <div>
                <label className="block text-sm font-medium mb-1">{label}</label>
                <p className="text-sm text-gray-500">{description}</p>
            </div>
            <ToggleSwitch checked={checked} onChange={onChange} />
        </div>
    );
};

export default SettingItem;