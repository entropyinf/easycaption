import React, { useState, useEffect } from 'react';

interface TextInputProps {
    label: string;
    value: string;
    description?: string;
    onChange: (value: string) => void;
    placeholder?: string;
    type?: string;
    disable?: boolean
}

const TextInput: React.FC<TextInputProps> = ({
    label,
    description,
    value,
    onChange,
    placeholder = '',
    disable = false,
    type = 'text',
}) => {
    const [localValue, setLocalValue] = useState<string>(value);

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setLocalValue(e.target.value);
    };

    const handleBlur = () => {
        if (localValue !== value) {
            onChange(localValue);
        }
    };

    useEffect(() => {
        setLocalValue(value);
    }, [value]);

    return (
        <div>
            <div>
                <label className="block text-sm font-medium mb-1 text-gray-900">{label}</label>
                {description && <p className="text-xs text-gray-500 mb-1">{description}</p>}
            </div>
            <div className="relative">
                <input
                    type={type}
                    value={localValue}
                    disabled={disable}
                    onChange={handleChange}
                    onBlur={handleBlur}
                    className={`w-full px-2.5 py-1.5 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-transparent transition duration-200 ease-in-out shadow-sm ${disable ? 'bg-gray-100 cursor-not-allowed opacity-75' : ''}`}
                    placeholder={placeholder}
                />
            </div>
        </div>
    );
};

export default TextInput;