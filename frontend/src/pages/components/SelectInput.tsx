import React from 'react';

interface SelectOption {
    value: string;
    label: string;
}

interface SelectInputProps {
    label: string;
    value: string;
    description?: string;
    onChange: (value: string) => void;
    options: SelectOption[];
}

const SelectInput: React.FC<SelectInputProps> = ({
    label,
    value,
    description,
    onChange,
    options
}) => {
    return (
        <div>
            <div>
                <label className="block text-sm font-medium mb-1">{label}</label>
                {description && <p className="text-sm text-gray-400">{description}</p>}
            </div>
            <select
                value={value}
                onChange={(e) => onChange(e.target.value)}
                style={{ appearance: 'none' }}
                className="w-full px-3 py-1 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
                {options.map((option, index) => (
                    <option
                        key={index}
                        value={option.value}
                    >
                        {option.label}
                    </option>
                ))}
            </select>
        </div>
    );
};

export default SelectInput;