import React from 'react';

interface NumberInputProps {
    label: string;
    value: number;
    onChange: (value: number) => void;
    placeholder?: string;
    step?: string;
}

const NumberInput: React.FC<NumberInputProps> = ({
    label,
    value,
    onChange,
    placeholder = '',
    step = '1'
}) => {
    return (
        <div>
            <label className="block text-sm font-medium mb-1">{label}</label>
            <input
                type="number"
                step={step}
                value={value}
                onChange={(e) => onChange(e.target.valueAsNumber || 0)}
                className="w-full px-3 py-1 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder={placeholder}
            />
        </div>
    );
};

export default NumberInput;