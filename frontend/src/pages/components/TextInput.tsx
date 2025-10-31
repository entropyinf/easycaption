import React from 'react';

interface TextInputProps {
    label: string;
    value: string;
    description?: string;
    onChange: (value: string) => void;
    placeholder?: string;
    type?: string;
}

const TextInput: React.FC<TextInputProps> = ({
    label,
    description,
    value,
    onChange,
    placeholder = '',
    type = 'text'
}) => {
    return (
        <div>
            <div>
                <label className="block text-sm font-medium mb-1">{label}</label>
                <p className="text-sm text-gray-500">{description}</p>
            </div>
            <input
                type={type}
                value={value}
                onChange={(e) => onChange(e.target.value)}
                className="w-full px-3 py-1 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder={placeholder}
            />
        </div>
    );
};

export default TextInput;